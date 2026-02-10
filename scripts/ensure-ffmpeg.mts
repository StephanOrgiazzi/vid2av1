import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import https from "node:https";
import http from "node:http";

const REQUIRED_BINARIES = ["ffmpeg.exe", "ffprobe.exe"] as const;
const DEFAULT_DOWNLOAD_URL =
  "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
const MAX_REDIRECTS = 5;

function resolveVendorBinDir(rootDir: string): string {
  return path.join(rootDir, "vendor", "ffmpeg", "bin");
}

function hasAllBinaries(binDir: string): boolean {
  return REQUIRED_BINARIES.every((name) => fs.existsSync(path.join(binDir, name)));
}

function ensureDirExists(dirPath: string): void {
  fs.mkdirSync(dirPath, { recursive: true });
}

function escapePowerShellLiteral(input: string): string {
  return input.replace(/'/g, "''");
}

function expandArchive(zipFilePath: string, destinationPath: string): void {
  const command = `Expand-Archive -LiteralPath '${escapePowerShellLiteral(zipFilePath)}' -DestinationPath '${escapePowerShellLiteral(destinationPath)}' -Force`;
  execFileSync(
    "powershell.exe",
    ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", command],
    { stdio: "inherit" }
  );
}

function downloadToFile(url: string, destinationPath: string, redirects = 0): Promise<void> {
  return new Promise((resolve, reject) => {
    const client = url.startsWith("https:") ? https : http;

    const request = client.get(url, (response) => {
      const statusCode = response.statusCode ?? 0;
      const location = response.headers.location;

      if (statusCode >= 300 && statusCode < 400 && location) {
        response.resume();
        if (redirects >= MAX_REDIRECTS) {
          reject(new Error(`Too many redirects while downloading FFmpeg from ${url}`));
          return;
        }

        const redirectedUrl = new URL(location, url).toString();
        downloadToFile(redirectedUrl, destinationPath, redirects + 1)
          .then(resolve)
          .catch(reject);
        return;
      }

      if (statusCode < 200 || statusCode >= 300) {
        response.resume();
        reject(new Error(`FFmpeg download failed with HTTP ${statusCode} from ${url}`));
        return;
      }

      const file = fs.createWriteStream(destinationPath);
      response.pipe(file);

      file.on("finish", () => {
        file.close();
        resolve();
      });

      file.on("error", (error) => {
        file.close();
        reject(error);
      });
    });

    request.on("error", reject);
  });
}

function findBinaryPath(searchRoot: string, binaryName: string): string | null {
  const entries = fs.readdirSync(searchRoot, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(searchRoot, entry.name);
    if (entry.isFile() && entry.name.toLowerCase() === binaryName.toLowerCase()) {
      return fullPath;
    }
  }

  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }

    const fullPath = path.join(searchRoot, entry.name);
    const nestedMatch = findBinaryPath(fullPath, binaryName);
    if (nestedMatch) {
      return nestedMatch;
    }
  }

  return null;
}

function copyRequiredBinaries(extractedDir: string, vendorBinDir: string): void {
  for (const binaryName of REQUIRED_BINARIES) {
    const sourcePath = findBinaryPath(extractedDir, binaryName);
    if (!sourcePath) {
      throw new Error(`Could not find ${binaryName} in extracted FFmpeg archive.`);
    }

    const targetPath = path.join(vendorBinDir, binaryName);
    fs.copyFileSync(sourcePath, targetPath);
    console.log(`Installed ${binaryName} -> ${targetPath}`);
  }
}

async function ensureFfmpegBinaries(rootDir: string): Promise<void> {
  const vendorBinDir = resolveVendorBinDir(rootDir);
  ensureDirExists(vendorBinDir);

  if (hasAllBinaries(vendorBinDir)) {
    console.log("FFmpeg binaries already present. Skipping download.");
    return;
  }

  if (process.platform !== "win32") {
    throw new Error(
      "Missing FFmpeg binaries and automatic setup currently supports Windows only."
    );
  }

  const downloadUrl = process.env.FFMPEG_DOWNLOAD_URL ?? DEFAULT_DOWNLOAD_URL;
  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vid2av1-ffmpeg-"));
  const archivePath = path.join(tempRoot, "ffmpeg.zip");
  const extractDir = path.join(tempRoot, "extract");
  ensureDirExists(extractDir);

  try {
    console.log(`Downloading FFmpeg from ${downloadUrl}`);
    await downloadToFile(downloadUrl, archivePath);
    console.log("Download complete. Extracting archive...");
    expandArchive(archivePath, extractDir);
    copyRequiredBinaries(extractDir, vendorBinDir);
    console.log("FFmpeg setup complete.");
  } finally {
    fs.rmSync(tempRoot, { recursive: true, force: true });
  }
}

async function main(): Promise<void> {
  await ensureFfmpegBinaries(process.cwd());
}

main().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  console.error(message);
  process.exit(1);
});
