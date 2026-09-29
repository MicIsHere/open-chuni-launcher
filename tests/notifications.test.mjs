import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import test from "node:test";
import ts from "typescript";

const source = await readFile(new URL("../src/composables/useNotifications.ts", import.meta.url), "utf8");
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.CommonJS },
});
const exports = {};
new Function("require", "exports", outputText)(createRequire(import.meta.url), exports);
const { notifications, notify, notifyError, dismiss, beginDismiss } = exports.useNotifications();

function reset() {
  notifications.value.forEach((item) => dismiss(item.id));
}

test("notification state is shared and closing removes only the selected card", () => {
  reset();
  const first = notify("success", "Saved");
  const second = notify("error", "Failed", { title: "Import failed" });
  assert.deepEqual(exports.useNotifications().notifications.value.map((item) => item.id), [first, second]);
  dismiss(first);
  dismiss(first);
  assert.deepEqual(notifications.value.map((item) => item.id), [second]);
});

test("identical errors are merged while different types remain separate", () => {
  reset();
  const id = notifyError(new Error("Disk full"), "Save failed");
  assert.equal(notifyError("Disk full", "Save failed"), id);
  notify("warning", "Disk full", { title: "Save failed" });
  assert.equal(notifications.value.length, 2);
  assert.equal(notifications.value[0].message, "Disk full");
});

test("the visible stack is bounded and durations follow severity or an explicit override", () => {
  reset();
  notify("success", "Success");
  notify("info", "Info");
  notify("warning", "Warning");
  notify("error", "Error");
  assert.deepEqual(notifications.value.map((item) => item.duration), [4000, 5000, 7000, 10000]);
  notify("info", "Persistent", { duration: 0 });
  notify("error", "Newest");
  assert.equal(notifications.value.length, 5);
  assert.equal(notifications.value[0].message, "Info");
  assert.equal(notifications.value.find((item) => item.message === "Persistent").duration, 0);
});

test("a repeated notification gets a new card when the previous card is exiting", () => {
  reset();
  const previous = notify("error", "Retry failed");
  beginDismiss(previous);
  const next = notify("error", "Retry failed");
  assert.notEqual(next, previous);
  dismiss(previous);
  assert.deepEqual(notifications.value.map((item) => item.id), [next]);
  assert.equal(notifications.value[0].closing, false);
});
