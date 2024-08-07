// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

// Utility functions for fetching data from a server using the Fetch API.
const chatToken = "TODO";
const chatUrl = "https://westus3.aqa.quantum.azure.com/api/chat/streaming";

const log = {
  debug: console.debug,
  error: console.error,
};

type quantumChatRequest = {
  conversationId: string; // GUID
  messages: Array<{
    role: string; // e.g. "user"
    content: string;
  }>; // The actual question
  additionalContext: any;
  identifier: string;
};

type QuantumChatResponse = {
  ConversationId: string; // GUID,
  Role: string; // e.g. "assistant"
  Content?: string; // The full response
  Delta?: string; // The next response token
  FinishReason?: string; // e.g. "stop"|"content_filter"|"length"|null,
  EmbeddedData: any;
  Created: string; // e.g. "2021-09-29T17:00:00.000Z"
};

// Consider using a generator function to handle the stream
export type CopilotStreamCallback = (mdFragment: string, done: boolean) => void;

export async function makeChatRequest(
  question: string,
  streamCallback: CopilotStreamCallback,
) {
  // log.debug("Starting copilot chat request flow");
  // const msaChatSession = await getAuthSession(
  //   [scopes.chatApi, `VSCODE_TENANT:common`, `VSCODE_CLIENT_ID:${chatApp}`],
  //   getRandomGuid(),
  // );
  // if (!msaChatSession) {
  //   throw Error("Failed to get MSA chat token");
  // }

  await chatRequest(chatToken, question, streamCallback);
}

// Guid format such as "00000000-1111-2222-3333-444444444444"
export function getRandomGuid(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(16));

  // Per https://www.ietf.org/rfc/rfc4122.txt, for UUID v4 (random GUIDs):
  // - Octet 6 contains the version in top 4 bits (0b0100)
  // - Octet 8 contains the variant in the top 2 bits (0b10)
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  // Convert the 16 bytes into 32 hex digits
  const hex = bytes.reduce(
    (acc, byte) => acc + byte.toString(16).padStart(2, "0"),
    "",
  );

  return (
    hex.substring(0, 8) +
    "-" +
    hex.substring(8, 12) +
    "-" +
    hex.substring(12, 16) +
    "-" +
    hex.substring(16, 20) +
    "-" +
    hex.substring(20, 32)
  );
}

async function chatRequest(
  token: string,
  question: string,
  streamCallback: CopilotStreamCallback,
  context?: string,
): Promise<any> {
  const payload: quantumChatRequest = {
    conversationId: getRandomGuid(),
    messages: [
      {
        role: "user",
        content: question,
      },
    ],
    additionalContext: {
      qcomEnvironment: "Desktop",
    },
    identifier: "Quantum",
  };

  if (context) {
    payload.messages.unshift({
      role: "assistant",
      content: context,
    });
  }

  const options = {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify(payload),
  };

  try {
    log.debug("About to call ChatAPI with payload: ", payload);
    await fetchEventSource(chatUrl, {
      ...options,
      onMessage(ev) {
        log.debug("Received copilot fetch message: ", ev);
        const messageReceived: QuantumChatResponse = JSON.parse(ev.data);
        log.debug("Received message: ", messageReceived);
        if (messageReceived.Delta) streamCallback(messageReceived.Delta, false);
      },
    });

    log.debug("ChatAPI fetch completed");
    streamCallback("", true);
    return Promise.resolve({});
  } catch (error) {
    log.error("ChatAPI fetch failed with error: ", error);
    throw error;
  }
}

/**
 * Represents a message sent in an event stream
 * https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#Event_stream_format
 */
export interface EventSourceMessage {
  /** The event ID to set the EventSource object's last event ID value. */
  id: string;
  /** A string identifying the type of event described. */
  event: string;
  /** The event data */
  data: string;
  /** The reconnection interval (in milliseconds) to wait before retrying the connection */
  retry?: number;
}

/**
 * Converts a ReadableStream into a callback pattern.
 * @param stream The input ReadableStream.
 * @param onChunk A function that will be called on each new byte chunk in the stream.
 * @returns {Promise<void>} A promise that will be resolved when the stream closes.
 */
export async function getBytes(
  stream: ReadableStream<Uint8Array>,
  onChunk: (arr: Uint8Array) => void,
) {
  const reader = stream.getReader();
  let result: ReadableStreamReadResult<Uint8Array>;
  while (!(result = await reader.read()).done) {
    onChunk(result.value);
  }
}

const enum ControlChars {
  NewLine = 10,
  CarriageReturn = 13,
  Space = 32,
  Colon = 58,
}

/**
 * Parses arbitrary byte chunks into EventSource line buffers.
 * Each line should be of the format "field: value" and ends with \r, \n, or \r\n.
 * @param onLine A function that will be called on each new EventSource line.
 * @returns A function that should be called for each incoming byte chunk.
 */
export function getLines(
  onLine: (line: Uint8Array, fieldLength: number) => void,
) {
  let buffer: Uint8Array | undefined;
  let position: number; // current read position
  let fieldLength: number; // length of the `field` portion of the line
  let discardTrailingNewline = false;

  // return a function that can process each incoming byte chunk:
  return function onChunk(arr: Uint8Array) {
    if (buffer === undefined) {
      buffer = arr;
      position = 0;
      fieldLength = -1;
    } else {
      // we're still parsing the old line. Append the new bytes into buffer:
      buffer = concat(buffer, arr);
    }

    const bufLength = buffer.length;
    let lineStart = 0; // index where the current line starts
    while (position < bufLength) {
      if (discardTrailingNewline) {
        if (buffer[position] === ControlChars.NewLine) {
          lineStart = ++position; // skip to next char
        }

        discardTrailingNewline = false;
      }

      // start looking forward till the end of line:
      let lineEnd = -1; // index of the \r or \n char
      for (; position < bufLength && lineEnd === -1; ++position) {
        switch (buffer[position]) {
          case ControlChars.Colon:
            if (fieldLength === -1) {
              // first colon in line
              fieldLength = position - lineStart;
            }
            break;
          case ControlChars.CarriageReturn:
            discardTrailingNewline = true;
            lineEnd = position;
            break;
          case ControlChars.NewLine:
            lineEnd = position;
            break;
        }
      }

      if (lineEnd === -1) {
        // We reached the end of the buffer but the line hasn't ended.
        // Wait for the next arr and then continue parsing:
        break;
      }

      // we've reached the line end, send it out:
      onLine(buffer.subarray(lineStart, lineEnd), fieldLength);
      lineStart = position; // we're now on the next line
      fieldLength = -1;
    }

    if (lineStart === bufLength) {
      buffer = undefined; // we've finished reading it
    } else if (lineStart !== 0) {
      // Create a new view into buffer beginning at lineStart so we don't
      // need to copy over the previous lines when we get the new arr:
      buffer = buffer.subarray(lineStart);
      position -= lineStart;
    }
  };
}

/**
 * Parses line buffers into EventSourceMessages.
 * @param onId A function that will be called on each `id` field.
 * @param onRetry A function that will be called on each `retry` field.
 * @param onMessage A function that will be called on each message.
 * @returns A function that should be called for each incoming line buffer.
 */
export function getMessages(
  onId: (id: string) => void,
  onRetry: (retry: number) => void,
  onMessage?: (msg: EventSourceMessage) => void,
) {
  let message = newMessage();
  const decoder = new TextDecoder();

  // return a function that can process each incoming line buffer:
  return function onLine(line: Uint8Array, fieldLength: number) {
    if (line.length === 0) {
      // empty line denotes end of message. Trigger the callback and start a new message:
      onMessage?.(message);
      message = newMessage();
    } else if (fieldLength > 0) {
      // exclude comments and lines with no values
      // line is of format "<field>:<value>" or "<field>: <value>"
      // https://html.spec.whatwg.org/multipage/server-sent-events.html#event-stream-interpretation
      const field = decoder.decode(line.subarray(0, fieldLength));
      const valueOffset =
        fieldLength + (line[fieldLength + 1] === ControlChars.Space ? 2 : 1);
      const value = decoder.decode(line.subarray(valueOffset));

      switch (field) {
        case "data":
          // if this message already has data, append the new value to the old.
          // otherwise, just set to the new value:
          message.data = message.data ? message.data + "\n" + value : value; // otherwise,
          break;
        case "event":
          message.event = value;
          break;
        case "id":
          onId((message.id = value));
          break;
        case "retry":
          {
            const retry = parseInt(value, 10);
            if (!isNaN(retry)) {
              // per spec, ignore non-integers
              onRetry((message.retry = retry));
            }
          }
          break;
      }
    }
  };
}

function concat(a: Uint8Array, b: Uint8Array) {
  const res = new Uint8Array(a.length + b.length);
  res.set(a);
  res.set(b, a.length);
  return res;
}

function newMessage(): EventSourceMessage {
  // data, event, and id must be initialized to empty strings:
  // https://html.spec.whatwg.org/multipage/server-sent-events.html#event-stream-interpretation
  // retry should be initialized to undefined so we return a consistent shape
  // to the js engine all the time: https://mathiasbynens.be/notes/shapes-ics#takeaways
  return {
    data: "",
    event: "",
    id: "",
    retry: undefined,
  };
}
export const EventStreamContentType = "text/event-stream";

const LastEventId = "last-event-id";

export interface FetchEventSourceInit extends RequestInit {
  /**
   * The request headers. FetchEventSource only supports the Record<string,string> format.
   */
  headers?: Record<string, string>;

  /**
   * Called when a message is received. NOTE: Unlike the default browser
   * EventSource.onmessage, this callback is called for _all_ events,
   * even ones with a custom `event` field.
   */
  onMessage?: (ev: EventSourceMessage) => void;
}

export function fetchEventSource(
  input: RequestInfo,
  {
    headers: inputHeaders,
    onMessage: onMessage,
    ...rest
  }: FetchEventSourceInit,
) {
  return new Promise<void>((resolve, reject) => {
    // make a copy of the input headers since we may modify it below:
    const headers = { ...inputHeaders };
    if (!headers.accept) {
      headers.accept = EventStreamContentType;
    }

    let curRequestController: AbortController;

    function dispose() {
      curRequestController.abort();
    }

    async function create() {
      curRequestController = new AbortController();
      try {
        const response = await fetch(input, {
          ...rest,
          headers,
          signal: curRequestController.signal,
        });

        await onOpen(response);

        await getBytes(
          response.body!,
          getLines(
            getMessages(
              (id) => {
                if (id) {
                  // store the id and send it back on the next retry:
                  headers[LastEventId] = id;
                } else {
                  // don't send the last-event-id header anymore:
                  delete headers[LastEventId];
                }
              },
              // eslint-disable-next-line @typescript-eslint/no-unused-vars
              (_retry) => {},
              onMessage,
            ),
          ),
        );

        dispose();
        resolve();
      } catch (err) {
        reject(err);
      }
    }

    create();
  });
}

function onOpen(response: Response) {
  const contentType = response.headers.get("content-type");
  if (!contentType?.startsWith(EventStreamContentType)) {
    throw new Error(
      `Expected content-type to be ${EventStreamContentType}, Actual: ${contentType}`,
    );
  }
}
