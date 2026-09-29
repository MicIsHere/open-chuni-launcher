import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import ts from "typescript";

const source = await readFile(new URL("../src/lib/plugins.ts", import.meta.url), "utf8");
const { outputText } = ts.transpileModule(source, {
  compilerOptions: { target: ts.ScriptTarget.ES2020, module: ts.ModuleKind.ESNext },
});
const { addPluginDlls, createPlugin, getGameDlls, loadPlugins } = await import(
  `data:text/javascript;base64,${Buffer.from(outputText).toString("base64")}`
);

test("launch includes the core DLL and only enabled plugins in order", () => {
  const plugins = loadPlugins();
  assert.deepEqual(getGameDlls(plugins), ["chusanhook.dll", "duolinguo.dll", "hook.dll"]);
  plugins[0].enabled = false;
  addPluginDlls(plugins, ["C:\\My Plugins\\extra.dll"], "Custom description");
  assert.deepEqual(getGameDlls(plugins), ["chusanhook.dll", "hook.dll", "C:\\My Plugins\\extra.dll"]);
  plugins.forEach((plugin) => { plugin.enabled = false; });
  assert.deepEqual(getGameDlls(plugins), ["chusanhook.dll"]);
});

test("legacy DLL lists migrate enabled states and retain custom paths", () => {
  const plugins = loadPlugins(undefined, ["chusanhook.dll", "HOOK.DLL", "C:\\Plugins\\extra.dll"]);
  assert.equal(plugins[0].enabled, false);
  assert.equal(plugins[1].enabled, true);
  assert.equal(plugins[2].dll, "C:\\Plugins\\extra.dll");
  assert.deepEqual(getGameDlls(loadPlugins(undefined, [])), ["chusanhook.dll"]);
});

test("saved plugin states and descriptions survive while built-in DLL names stay fixed", () => {
  const plugins = loadPlugins([
    { id: "duolinguo", dll: "renamed.dll", enabled: false, description: "Edited description" },
    { id: "hook", dll: "C:\\Plugins\\HOOK.DLL", enabled: true, description: "Hook notes" },
    { dll: "/plugins/custom.dll", enabled: false, description: "Custom notes" },
  ]);
  assert.equal(plugins.length, 3);
  assert.deepEqual(plugins[0], {
    id: "duolinguo", dll: "duolinguo.dll", enabled: false, description: "Edited description",
  });
  assert.equal(plugins[1].dll, "hook.dll");
  assert.deepEqual(loadPlugins(JSON.parse(JSON.stringify(plugins))), plugins);
});

test("imports skip duplicate DLL names, built-in DLLs and the core DLL", () => {
  const plugins = loadPlugins();
  const count = addPluginDlls(plugins, [
    "C:\\Plugins\\hook.dll", "C:\\Plugins\\duolinguo.dll", "chusanhook.dll",
    "/first/EXTRA.DLL", "/second/extra.dll", "/plugins/another.dll",
  ]);
  assert.equal(count, 2);
  assert.deepEqual(plugins.slice(2).map((plugin) => plugin.dll), ["/first/EXTRA.DLL", "/plugins/another.dll"]);
});

test("invalid DLLs and malformed storage are ignored", () => {
  for (const dll of ["", " ", "notes.txt", "folder.dll/", ".dll"]) {
    assert.equal(createPlugin(dll), undefined);
  }
  assert.equal(createPlugin("  /plugins/Extra.DLL  ").dll, "/plugins/Extra.DLL");
  assert.equal(loadPlugins([null, true, "bad.dll", { dll: 123 }, { dll: "notes.txt" }]).length, 2);
  assert.equal(loadPlugins({}, "invalid").length, 2);
});
