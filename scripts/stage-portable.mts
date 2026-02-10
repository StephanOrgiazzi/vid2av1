import fs from "node:fs";
import path from "node:path";

const PORTABLE_BINARIES = ["ffmpeg.exe", "ffprobe.exe"] as const;

interface StagePaths {
  ffmpegDir: string;
  releaseDir: string;
}

function resolvePaths(rootDir: string): StagePaths {
  return {
    ffmpegDir: path.join(rootDir, "vendor", "ffmpeg", "bin"),
    releaseDir: path.join(rootDir, "src-tauri", "target", "release")
  };
}

function assertReleaseDirExists(releaseDir: string): void {
  if (!fs.existsSync(releaseDir)) {
    throw new Error(`Release directory not found: ${releaseDir}`);
  }
}

function stagePortableDependencies(paths: StagePaths): number {
  let copiedCount = 0;

  for (const fileName of PORTABLE_BINARIES) {
    const sourcePath = path.join(paths.ffmpegDir, fileName);
    const destinationPath = path.join(paths.releaseDir, fileName);

    if (!fs.existsSync(sourcePath)) {
      console.warn(`Skipping missing dependency: ${sourcePath}`);
      continue;
    }

    fs.copyFileSync(sourcePath, destinationPath);
    copiedCount += 1;
    console.log(`Copied ${fileName} -> ${destinationPath}`);
  }

  return copiedCount;
}

function main(): void {
  const paths = resolvePaths(process.cwd());
  assertReleaseDirExists(paths.releaseDir);

  const copiedCount = stagePortableDependencies(paths);
  if (copiedCount === 0) {
    throw new Error("No FFmpeg binaries were copied. Portable package is incomplete.");
  }

  console.log("Portable dependencies staged.");
}

try {
  main();
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  console.error(message);
  process.exit(1);
}
