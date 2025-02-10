// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { ChatElement } from "../shared";
import { useRef, useState } from "preact/hooks";
import { Markdown } from "qsharp-lang/ux";

/**
 * A button that appears below each user message
 * that allows resubmitting the message, with an
 * option to change the service backend.
 */
export function RetryButton(props: {
  history: ChatElement[];
  serviceOptions: string[];
  selectedService?: string;
  restartChat: (history: ChatElement[], service: string) => void;
  disabled: boolean;
}) {
  const serviceDropdown = useRef<HTMLSelectElement>(null);
  return (
    <div className="retry-button">
      <select ref={serviceDropdown} value={props.selectedService}>
        {props.serviceOptions.map((service) => (
          <option value={service}>{service}</option>
        ))}
      </select>
      <button
        onClick={() => {
          const selectedService = serviceDropdown.current!.value;
          props.restartChat(props.history, selectedService);
        }}
        disabled={props.disabled}
      >
        Retry
      </button>
    </div>
  );
}

/**
 * Displays a tool call in the chat history.
 */
export function ToolMessage(props: {
  id: string;
  content: string;
  history: ChatElement[];
}) {
  const { name, args } = lookupToolCall(props.history, props.id);
  let content = props.content;
  try {
    // reformat JSON to be readable
    const parsedContent = JSON.parse(content);
    content = JSON.stringify(parsedContent, undefined, 2);
  } catch {
    // content wasn't valid JSON, no need to reformat it
  }

  const [shownContent, setShownContent] = useState(
    content.length > 120
      ? content.slice(0, 100) + "... (click to see more)"
      : content,
  );

  return (
    // match the widget style from https://github.com/microsoft/vscode/blob/c799d209cd4846a2a822b55dbf2ca21893008faa/src/vs/workbench/contrib/chat/browser/media/chatCodeBlockPill.css#L6
    <div className="left-message-row">
      <div
        style="
            border: 1px solid var(--vscode-chat-requestBorder, var(--vscode-input-background, transparent));
          border-radius: 4px;
          width: fit-content;
          font-weight: normal;
          text-decoration: none;
          font-size: 11px;
          padding: 0 3px;
          white-space: pre;
        "
      >
        {name}({args}) =&gt;{" "}
        <span onClick={() => setShownContent(content)}>{shownContent}</span>
      </div>
    </div>
  );
}

/**
 * For display purposes, find the name and arguments for a tool call in the chat history.
 */
function lookupToolCall(
  history: ChatElement[],
  id: string,
): { name: string; args: string } {
  const toolCalls = history.flatMap((m) =>
    m.role === "assistant" ? (m.ToolCalls ?? []) : [],
  );
  const call = toolCalls.find((tc) => tc.id === id);
  return {
    name: call?.name ?? "unknown",
    args: JSON.stringify(call?.arguments ?? {}),
  };
}

/**
 * Views the chat request payload that would get sent to the service in the
 * next request. This would contain the current chat history.
 */
export function ShowPayload(props: {
  history: ChatElement[];
  service?: string;
  serviceOptions: string[];
  restartChat: (history: ChatElement[], service: string) => void;
}) {
  const payloadRef = useRef<HTMLDivElement>(null);
  const messages = props.history.filter((m) => m.role !== "widget");
  return (
    <>
      <div>
        <a
          href="#"
          onClick={(e) => {
            e.preventDefault();
            if (payloadRef.current) {
              payloadRef.current.style.display =
                payloadRef.current.style.display === "none" ? "block" : "none";
            }
          }}
        >
          chat request payload
        </a>
        <div ref={payloadRef} style="display: none;">
          <a
            href="#"
            onClick={(e) => {
              e.preventDefault();
              navigator.clipboard.writeText(
                JSON.stringify(props.history, undefined, 2),
              );
              if (e.target) {
                const a = e.target as HTMLAnchorElement;
                a.textContent = "copy to clipboard - copied.";
                setTimeout(() => {
                  a.textContent = "copy to clipboard";
                }, 2000);
              }
            }}
          >
            copy to clipboard
          </a>
          <Markdown
            markdown={
              "```json\n" + JSON.stringify(messages, undefined, 2) + "\n```"
            }
          ></Markdown>
        </div>
      </div>
    </>
  );
}

/**
 * Radio buttons to switch to a different service backend for testing.
 */
export function ServiceSelector(props: {
  service?: string;
  serviceOptions: string[];
  restartChat: (history: ChatElement[], service: string) => void;
}) {
  return (
    <div className="service-radio-group">
      {props.serviceOptions.map((service) => (
        <label>
          <input
            type="radio"
            name="service"
            value={service}
            checked={props.service === service}
            onChange={(e) => props.restartChat([], (e as any).target.value)}
          />
          {service}
        </label>
      ))}
    </div>
  );
}
