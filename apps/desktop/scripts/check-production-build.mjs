import { readdirSync, readFileSync } from "node:fs";
import { join, relative, resolve, sep } from "node:path";
import { build } from "vite";

const project = resolve(import.meta.dirname, "..");
const tauri = JSON.parse(readFileSync(join(project, "src-tauri", "tauri.conf.json"), "utf8"));
const outDir = resolve(project, "src-tauri", tauri.build.frontendDist);
const forbidden = [join(project, "src", "dev-preview") + sep, join(project, "src", "test") + sep];
const MARKERS = ["Interface preview", "interface preview", "PrismForge fake OpenRGB", "store-folder/demo-store"];

function files(folder) {
  return readdirSync(folder, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? files(join(folder, entry.name)) : [join(folder, entry.name)],
  );
}

const modules = [];
const record = {
  name: "record-modules",
  generateBundle() {
    modules.push(...this.getModuleIds());
  },
};

await build({ root: project, configFile: join(project, "vite.config.ts"), logLevel: "silent", plugins: [record], build: { outDir, emptyOutDir: true } });

const local = modules.map((id) => resolve(id.split("?")[0].replace(/^\0/, "")));
const leaked = local.filter((id) => forbidden.some((folder) => id.startsWith(folder))).map((id) => relative(project, id));
const built = files(outDir);
const text = built.map((file) => readFileSync(file, "utf8")).join("\n");
const marked = MARKERS.filter((marker) => text.includes(marker));
const problems = [
  ...leaked.map((id) => `module ${id}`),
  ...marked.map((marker) => `text "${marker}"`),
  ...(local.some((id) => id === join(project, "src", "main.ts")) ? [] : ["the build did not start from src/main.ts"]),
];
if (problems.length > 0) {
  console.error(`The production build in ${relative(project, outDir)} holds preview or test code: ${problems.join(", ")}`);
  process.exitCode = 1;
} else {
  console.log(
    `The production build in ${relative(project, outDir)} holds no module from src/dev-preview or src/test (${local.length} modules, ${built.length} files checked).`,
  );
}
