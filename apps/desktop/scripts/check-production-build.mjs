import { mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative, resolve } from "node:path";
import { build } from "vite";

const project = resolve(import.meta.dirname, "..");
const outDir = mkdtempSync(join(tmpdir(), "prismforge-production-"));

const MARKERS = [
  "interface preview",
  "Interface preview",
  "PrismForge fake OpenRGB",
  "store-folder/demo-store",
  "dev-1052f35afd339289",
  "Simulated memory slot 1",
];

function files(folder) {
  return readdirSync(folder, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? files(join(folder, entry.name)) : [join(folder, entry.name)],
  );
}

try {
  await build({ root: project, configFile: join(project, "vite.config.ts"), logLevel: "silent", build: { outDir, emptyOutDir: true } });
  const built = files(outDir);
  const names = built.map((file) => relative(outDir, file));
  const text = built.map((file) => readFileSync(file, "utf8")).join("\n");
  const problems = [
    ...names.filter((name) => /preview|fixture/i.test(name)).map((name) => `file ${name}`),
    ...MARKERS.filter((marker) => text.includes(marker)).map((marker) => `text "${marker}"`),
    ...(names.includes("index.html") && text.includes("PrismForge") ? [] : ["no index.html with the app"]),
  ];
  if (problems.length > 0) {
    console.error(`The production build holds preview or fixture content: ${problems.join(", ")}`);
    process.exitCode = 1;
  } else {
    console.log(`The production build holds no preview code and no fixture data (${names.length} files checked).`);
  }
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
