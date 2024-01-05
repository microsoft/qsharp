// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import { useRef, useState } from "preact/hooks";
import { type ReData } from "./reTable.js";

type CellValue = string | number | { value: string; sortBy: number };

// Note: column 0 is expected to be unique amongst all rows
export function ResultsTable(props: {
  columnNames: string[];
  data: ReData[];
  initialColumns: number[];
  ensureSelected: boolean;
  onRowSelected(rowId: string): void;
  onRowDeleted(rowId: string): void;
}) {
  const [showColumns, setShowColumns] = useState(props.initialColumns);
  const [sortColumn, setSortColumn] = useState<{
    columnId: number;
    ascending: boolean;
  } | null>(null);
  const [selectedRow, setSelectedRow] = useState<string>("");
  const [showColumnMenu, setShowColumnMenu] = useState(false);
  const [showRowMenu, setShowRowMenu] = useState("");

  const rows = props.data.map(ReDataToRow);

  // Find the first row that is new in the current sort order
  const newest = getSortedRows(rows).find(
    (row) => (row[row.length - 1] as string) === "New",
  );

  // Select the first of the newest rows, otherwise preserve the existing selection
  if (newest && props.ensureSelected) {
    const rowId = newest[0].toString();
    setSelectedRow(rowId);
    props.onRowSelected(rowId);
  } else if (!selectedRow && props.ensureSelected && rows.length > 0) {
    const rowId = rows[0][0].toString();
    setSelectedRow(rowId);
    props.onRowSelected(rowId);
  }

  // Use to track the column being dragged
  const draggingCol = useRef("");

  /*
  Note: Drag and drop events can occur faster than preact reconciles state.
  This causes challenges where one event will set state, and the next event
  that needs to use the latest state will still see old state.

  So don't apply state changes in the drag and drop handlers. Instead, set
  styles directly, and just update state if the 'drop' event changes column
  order.
  */

  function onDragStart(ev: DragEvent) {
    if (!(ev.target instanceof HTMLElement)) return;
    const colid = ev.target.closest("th")?.dataset["colid"];
    draggingCol.current = colid!;
    ev.dataTransfer!.dropEffect = "move";
  }

  function onDragEnter(ev: DragEvent) {
    // Get the column id of the column being entered
    if (!(ev.target instanceof HTMLElement)) return;
    const thisColId = ev.target.closest("th")?.dataset["colid"];
    if (!thisColId || !draggingCol.current) return;

    // If this column is different to the column being dragged, add the CSS class
    if (draggingCol.current !== thisColId) {
      ev.preventDefault();
      ev.dataTransfer!.dropEffect = "move";
      ev.target
        .closest("table")!
        .querySelectorAll(`[data-colid="${thisColId}"]`)
        .forEach((elem) => elem.classList.add("qs-resultsTable-dragEnter"));
    }
  }

  function onDragOver(ev: DragEvent) {
    if (!(ev.target instanceof HTMLElement)) return;
    const thisColId = ev.target.closest("th")?.dataset["colid"];
    if (!thisColId || !draggingCol.current) return;

    // If dragging something over a different column, allow the drop
    if (draggingCol.current !== thisColId) {
      ev.dataTransfer!.dropEffect = "move";
      ev.preventDefault();
    }
  }

  function onDragLeave(ev: DragEvent) {
    // Remove the CSS class from the column being left
    if (!(ev.target instanceof HTMLElement)) return;
    const thisColId = ev.target.closest("th")?.dataset["colid"];
    if (!thisColId) return;

    ev.target
      .closest("table")!
      .querySelectorAll(`[data-colid="${thisColId}"]`)
      .forEach((elem) => elem.classList.remove("qs-resultsTable-dragEnter"));
    ev.preventDefault();
  }

  function onDrop(ev: DragEvent) {
    if (!(ev.target instanceof HTMLElement)) return;
    const thisColId = ev.target.closest("th")?.dataset["colid"];
    if (!thisColId) return;

    if (draggingCol.current) {
      moveColumn(parseInt(draggingCol.current), parseInt(thisColId));
      ev.preventDefault();
    }
  }

  function onDragEnd(ev: DragEvent) {
    // Called regardless of how dragging ends
    // ev.target is the source element
    // Just remove any dragEnter classes from cells that may remain
    (ev.target as HTMLElement)
      .closest("table")!
      .querySelectorAll(`th, td`)
      .forEach((elem) => elem.classList.remove("qs-resultsTable-dragEnter"));
    draggingCol.current = "";
  }

  function moveColumn(oldIdx: number, newIdx: number) {
    // Locate the indexes in the array where the column idx is
    const arrIdxOld = showColumns.indexOf(oldIdx);
    const arrIdxNew = showColumns.indexOf(newIdx);

    const newColumns = [...showColumns];
    const removed = newColumns.splice(arrIdxOld, 1);
    newColumns.splice(arrIdxNew, 0, ...removed);
    setShowColumns(newColumns);
  }

  function onSort(ev: MouseEvent) {
    if (!(ev.currentTarget instanceof HTMLTableCellElement)) return;
    const thisCol = ev.currentTarget.dataset["colid"];

    if (sortColumn && thisCol === sortColumn.columnId.toString()) {
      // Toggle the sort order
      setSortColumn({
        columnId: sortColumn.columnId,
        ascending: !sortColumn.ascending,
      });
    } else {
      // Set the sort column
      setSortColumn({
        columnId: parseInt(thisCol!),
        ascending: true,
      });
    }
  }

  function getSortedRows(rows: CellValue[][]) {
    if (!sortColumn) return rows;

    const colIdx = sortColumn.columnId;
    const ascending = sortColumn.ascending;

    const sortedRows = [...rows];
    sortedRows.sort((a, b) => {
      const aVal = a[colIdx];
      const bVal = b[colIdx];
      if (typeof aVal === "string" && typeof bVal === "string") {
        return ascending ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal);
      } else if (typeof aVal === "number" && typeof bVal === "number") {
        return ascending ? aVal - bVal : bVal - aVal;
      } else if (typeof aVal === "object" && typeof bVal === "object") {
        return ascending
          ? aVal.sortBy - bVal.sortBy
          : bVal.sortBy - aVal.sortBy;
      } else {
        return 0;
      }
    });

    return sortedRows;
  }

  function getCellStr(col: CellValue) {
    if (typeof col === "object") {
      return col.value;
    } else if (typeof col === "number") {
      return col.toLocaleString();
    } else {
      return col || "";
    }
  }

  function rowClicked(rowId: string) {
    if (selectedRow === rowId && props.ensureSelected) return;

    const newSelectedRow = selectedRow === rowId ? "" : rowId;
    setSelectedRow(newSelectedRow);
    props.onRowSelected(newSelectedRow);
  }

  function onClickRowMenu(ev: MouseEvent, rowid: string) {
    ev.stopPropagation();
    if (showRowMenu === rowid) {
      setShowRowMenu("");
    } else {
      setShowRowMenu(rowid!);
    }
  }

  function onClickColumnMenu(ev: MouseEvent) {
    ev.stopPropagation();
    setShowRowMenu("");
    setShowColumnMenu(!showColumnMenu);
  }

  function getColumnList() {
    return props.columnNames.map((name, idx) => ({
      name,
      idx,
      show: showColumns.includes(idx),
    }));
  }

  function toggleColumn(idx: number) {
    const newColumns = [...showColumns];
    const arrIdx = newColumns.indexOf(idx);
    if (arrIdx === -1) {
      // Not currently showing, need to add it
      if (idx > newColumns.length) {
        // The column position is greater than the number of columns currently showing
        // So just add to the end
        newColumns.push(idx);
      } else {
        // Insert at the correct position
        newColumns.splice(idx, 0, idx);
      }
    } else {
      newColumns.splice(arrIdx, 1);
    }
    setShowColumns(newColumns);
  }

  function deleteRow(e: MouseEvent, rowId: string) {
    e.stopPropagation();
    // Clear out any menus or selections for the row if needed
    setShowRowMenu("");
    if (selectedRow === rowId) {
      setSelectedRow("");
      props.onRowSelected("");
    }
    props.onRowDeleted(rowId);
  }

  return (
    <table class="qs-resultsTable-sortedTable">
      <thead>
        <tr>
          <th>
            <div style="position: relative">
              <svg
                width="16"
                height="16"
                style="position: relative;"
                onClick={onClickColumnMenu}
              >
                <rect x="1" y="3.5" width="14" height="2" fill="black" />
                <rect
                  x="1"
                  y="3"
                  width="14"
                  height="12"
                  stroke="gray"
                  stroke-width="1"
                  fill="none"
                  rx="2"
                />
                <path
                  stroke="gray"
                  stroke-width="1"
                  d="M4.5,3 V15 M8,3 V15 M11.5,3 V15"
                />
              </svg>
              <div
                class={
                  showColumnMenu
                    ? "qs-resultsTable-columnMenu qs-resultsTable-showColumnMenu"
                    : "qs-resultsTable-columnMenu"
                }
                style="position: absolute; top: 16; left: 0;"
              >
                {getColumnList().map((elem) => (
                  <div
                    width="100px"
                    height="20px"
                    class={
                      elem.show
                        ? "qs-resultsTable-columnSelected"
                        : "qs-resultsTable-menuItem"
                    }
                    onClick={() => toggleColumn(elem.idx)}
                  >
                    {elem.name}
                  </div>
                ))}
              </div>
            </div>
          </th>
          {showColumns.map((idx) => {
            const isSortColumn = sortColumn?.columnId === idx;
            return (
              <th onClick={onSort} data-colid={idx.toString()}>
                <span
                  class={
                    isSortColumn
                      ? "qs-resultsTable-sortHeaderCell"
                      : "qs-resultsTable-headerCell"
                  }
                  draggable
                  onDragStart={onDragStart}
                  onDragEnter={onDragEnter}
                  onDragOver={onDragOver}
                  onDragLeave={onDragLeave}
                  onDragEnd={onDragEnd}
                  onDrop={onDrop}
                >
                  {props.columnNames[idx]}
                </span>
                {isSortColumn ? (
                  <svg
                    width="16"
                    height="16"
                    style={`transform: rotate(${
                      sortColumn!.ascending ? "0" : "180"
                    }deg)`}
                  >
                    <polygon fill="gray" points="2,10 8,4 14,10" />
                  </svg>
                ) : null}
              </th>
            );
          })}
        </tr>
      </thead>
      <tbody>
        {getSortedRows(rows).map((row) => {
          const rowId = row[0].toString();
          return (
            <tr
              onClick={() => rowClicked(rowId)}
              data-rowid={rowId}
              class={
                rowId === selectedRow
                  ? "qs-resultsTable-sortedTableSelectedRow"
                  : undefined
              }
            >
              <td>
                <div
                  style="position: relative"
                  onClick={(e) => onClickRowMenu(e, rowId)}
                >
                  <svg width="16" height="16" style="position: relative;">
                    <path
                      stroke-width="1.5"
                      stroke="gray"
                      stroke-linecap="round"
                      d="M4,5 h8 M4,8 h8 M4,11 h8"
                    />
                  </svg>
                  {showRowMenu === rowId ? (
                    <div
                      class="qs-resultsTable-showColumnMenu"
                      style="top: 16px; left: 0px;"
                    >
                      <div
                        class="qs-resultsTable-menuItem"
                        onClick={(e) => deleteRow(e, rowId)}
                      >
                        Delete
                      </div>
                    </div>
                  ) : null}
                </div>
              </td>
              {showColumns.map((idx) => {
                return (
                  <td data-colid={idx.toString()}>{getCellStr(row[idx])}</td>
                );
              })}
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}

function ReDataToRow(data: ReData): CellValue[] {
  return [
    data.jobParams.runName,
    data.jobParams.qubitParams.name,
    data.jobParams.qecScheme.name,
    data.jobParams.errorBudget,
    data.physicalCounts.breakdown.algorithmicLogicalQubits,
    data.physicalCounts.breakdown.algorithmicLogicalDepth,
    data.logicalQubit.codeDistance,
    data.physicalCounts.breakdown.numTstates,
    data.physicalCounts.breakdown.numTfactories,
    data.physicalCountsFormatted.physicalQubitsForTfactoriesPercentage,
    {
      value: data.physicalCountsFormatted.runtime,
      sortBy: data.physicalCounts.runtime,
    },
    data.physicalCounts.rqops,
    data.physicalCounts.physicalQubits,
    data.new ? "New" : "Cached",
  ];
}
