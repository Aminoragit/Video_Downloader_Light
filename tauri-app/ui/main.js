const invoke = window.__TAURI__.core.invoke;
const listen = window.__TAURI__.event.listen;

const elements = {
  ytdlpStatus: document.querySelector("#ytdlpStatus"),
  ffmpegStatus: document.querySelector("#ffmpegStatus"),
  aria2cStatus: document.querySelector("#aria2cStatus"),
  ytdlpDot: document.querySelector("#ytdlpDot"),
  ffmpegDot: document.querySelector("#ffmpegDot"),
  aria2cDot: document.querySelector("#aria2cDot"),
  refreshStatus: document.querySelector("#refreshStatus"),
  downloadButton: document.querySelector("#downloadButton"),
  url: document.querySelector("#url"),
  saveDir: document.querySelector("#saveDir"),
  chooseSaveDir: document.querySelector("#chooseSaveDir"),
  mode: document.querySelector("#mode"),
  quality: document.querySelector("#quality"),
  audioQuality: document.querySelector("#audioQuality"),
  statusText: document.querySelector("#statusText"),
  progressText: document.querySelector("#progressText"),
  progressBar: document.querySelector("#progressBar"),
  installDetail: document.querySelector("#installDetail"),
  log: document.querySelector("#log"),
  saveSettings: document.querySelector("#saveSettings"),
  historyList: document.querySelector("#historyList"),
  clearHistory: document.querySelector("#clearHistory"),
  legalOverlay: document.querySelector("#legalOverlay"),
  acceptLegal: document.querySelector("#acceptLegal"),
  downloadThumbnail: document.querySelector("#downloadThumbnail"),
  thumbnailPlaceholder: document.querySelector("#thumbnailPlaceholder"),
  downloadTitle: document.querySelector("#downloadTitle"),
  downloadDuration: document.querySelector("#downloadDuration"),
  downloadSize: document.querySelector("#downloadSize")
};

const pageButtons = Array.from(document.querySelectorAll("[data-page-target]"));
const pagePanels = Array.from(document.querySelectorAll("[data-page]"));
const adCards = Array.from(document.querySelectorAll(".ad-card"));
const donationButtons = Array.from(document.querySelectorAll("[data-open-url]"));

const runtimeElements = {
  ytdlp: { status: elements.ytdlpStatus, dot: elements.ytdlpDot },
  ffmpeg: { status: elements.ffmpegStatus, dot: elements.ffmpegDot },
  aria2c: { status: elements.aria2cStatus, dot: elements.aria2cDot }
};

const installingComponents = new Set();
const failedComponents = new Set();
const componentLabels = {
  ytdlp: "yt-dlp",
  ffmpeg: "ffmpeg",
  aria2c: "aria2c"
};
const INSTALLING_MESSAGE = "필수 파일을 다운로드 중입니다. 잠시만 기다려 주시기 바랍니다.";
const validModes = new Set(["all", "video", "audio"]);
const validVideoQualities = new Set(["best", "2160", "1440", "1080", "720", "480", "360"]);
const validAudioQualities = new Set(["best", "320", "256", "192", "128"]);

let autoInstallStarted = false;
let runtimeInstallPlan = [];
let activePage = "download";

function appendLog(message) {
  elements.log.textContent += `${message}\n`;
  elements.log.scrollTop = elements.log.scrollHeight;
}

function showPage(pageName) {
  activePage = pageName;

  pagePanels.forEach((panel) => {
    const isActive = panel.dataset.page === pageName;
    panel.hidden = !isActive;
    panel.setAttribute("aria-hidden", String(!isActive));
  });

  pageButtons.forEach((button) => {
    const isActive = button.dataset.pageTarget === pageName;
    button.classList.toggle("active", isActive);
    button.setAttribute("aria-selected", String(isActive));
    button.tabIndex = isActive ? 0 : -1;
  });
}

function setBusy(isBusy) {
  elements.downloadButton.disabled = isBusy;
  elements.refreshStatus.disabled = isBusy;
  elements.saveSettings.disabled = isBusy;
  elements.chooseSaveDir.disabled = isBusy;
  elements.saveDir.disabled = isBusy;
}

function setProgress(percent, message) {
  const value = Math.max(0, Math.min(100, Number(percent) || 0));
  elements.progressBar.style.width = `${value}%`;
  elements.progressText.textContent = `${value.toFixed(1)}%`;
  if (message) elements.statusText.textContent = message;
}

function setInstallDetail(message, show = true) {
  elements.installDetail.textContent = message || "";
  elements.installDetail.hidden = !show;
}

function setDownloadMetadata(metadata = {}) {
  const title = metadata.title || "다운로드 대기 중";
  const duration = metadata.duration || "-";
  const rawSize = metadata.size || "-";
  const size = metadata.sizeApproximate && rawSize !== "-" && rawSize !== "계산 중" ? `${rawSize} 예상` : rawSize;
  const thumbnail = metadata.thumbnail || "";

  elements.downloadTitle.textContent = title;
  elements.downloadDuration.textContent = duration;
  elements.downloadSize.textContent = size;

  if (thumbnail) {
    elements.downloadThumbnail.src = thumbnail;
    elements.downloadThumbnail.hidden = false;
    elements.thumbnailPlaceholder.hidden = true;
  } else {
    elements.downloadThumbnail.removeAttribute("src");
    elements.downloadThumbnail.hidden = true;
    elements.thumbnailPlaceholder.hidden = false;
  }
}

function resetDownloadMetadata(status = "다운로드 대기 중") {
  setDownloadMetadata({ title: status, duration: "-", size: "-", thumbnail: "" });
}

function componentLabel(name) {
  return componentLabels[name] || name;
}

function runtimeOverallPercent(name, componentPercent) {
  if (!runtimeInstallPlan.length) return componentPercent;
  const index = Math.max(0, runtimeInstallPlan.indexOf(name));
  return ((index + Math.max(0, Math.min(100, Number(componentPercent) || 0)) / 100) / runtimeInstallPlan.length) * 100;
}

function setRuntimeDot(dot, state) {
  dot.classList.remove("ok", "pending", "fail");
  dot.classList.add(state);
}

function renderComponentStatus(name, component) {
  const target = runtimeElements[name];
  if (!target) return;

  if (installingComponents.has(name)) {
    target.status.textContent = "설치 중";
    setRuntimeDot(target.dot, "pending");
    return;
  }

  if (component.installed) {
    target.status.textContent = "설치됨";
    setRuntimeDot(target.dot, "ok");
    return;
  }

  target.status.textContent = failedComponents.has(name) ? "실패" : "미설치";
  setRuntimeDot(target.dot, "fail");
}

function currentSettings() {
  return {
    saveDir: elements.saveDir.value.trim(),
    mode: elements.mode.value,
    quality: elements.quality.value,
    audioQuality: elements.audioQuality.value,
    audioFormat: "original"
  };
}

function applySettings(settings) {
  if (!settings) return;

  const mode = validModes.has(settings.mode) ? settings.mode : "all";
  const quality = validVideoQualities.has(settings.quality) ? settings.quality : "best";
  const audioQuality = validAudioQualities.has(settings.audioQuality) ? settings.audioQuality : "best";

  elements.saveDir.value = settings.saveDir || elements.saveDir.value;
  elements.mode.value = mode;
  elements.quality.value = quality;
  elements.audioQuality.value = audioQuality;
  elements.legalOverlay.hidden = Boolean(settings.legalAccepted);
}

function renderHistory(records = []) {
  elements.historyList.textContent = "";

  if (!records.length) {
    const empty = document.createElement("p");
    empty.className = "hint empty-state";
    empty.textContent = "다운로드 이력이 없습니다.";
    elements.historyList.append(empty);
    return;
  }

  for (const record of records) {
    const item = document.createElement("article");
    item.className = "history-item";

    const body = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = record.url;
    const meta = document.createElement("span");
    meta.textContent = `${record.status} | ${record.requestedAt} | ${record.outputDir}`;
    body.append(title, meta);

    const actions = document.createElement("div");
    actions.className = "history-actions";

    const requestAgain = document.createElement("button");
    requestAgain.className = "ghost small-button";
    requestAgain.type = "button";
    requestAgain.textContent = "새로 다운로드 요청";
    requestAgain.addEventListener("click", () => {
      elements.url.value = record.url;
      showPage("download");
      elements.url.focus();
      appendLog("[정보] 과거 URL을 입력칸에 불러왔습니다. 다운로드 버튼을 눌러야 실제 작업이 시작됩니다.");
    });

    const remove = document.createElement("button");
    remove.className = "ghost small-button danger-button";
    remove.type = "button";
    remove.textContent = "삭제";
    remove.addEventListener("click", () => deleteHistory(record.id));

    actions.append(requestAgain, remove);
    item.append(body, actions);
    elements.historyList.append(item);
  }
}

function missingComponents(state) {
  return [
    ["ytdlp", state.ytdlp],
    ["ffmpeg", state.ffmpeg],
    ["aria2c", state.aria2c]
  ].filter(([, component]) => !component.installed);
}

async function refreshStatus() {
  const state = await invoke("get_state");
  renderComponentStatus("ytdlp", state.ytdlp);
  renderComponentStatus("ffmpeg", state.ffmpeg);
  renderComponentStatus("aria2c", state.aria2c);
  applySettings({ ...state.settings, saveDir: state.settings.saveDir || state.defaultSaveDir });
  renderHistory(state.history);
  return state;
}

async function saveCurrentSettings(showToast = true) {
  await invoke("save_settings", { settings: currentSettings() });
  if (showToast) appendLog("[정보] 설정을 저장했습니다.");
}

async function installComponent(name) {
  installingComponents.add(name);
  failedComponents.delete(name);
  renderComponentStatus(name, { installed: false });
  appendLog(`[정보] ${componentLabel(name)} 설치를 시작합니다.`);
  setProgress(runtimeOverallPercent(name, 0), INSTALLING_MESSAGE);
  setInstallDetail(`${componentLabel(name)}: 설치 준비 중`);

  try {
    const result = await invoke("install_component", { name });
    appendLog(result);
    installingComponents.delete(name);
    await refreshStatus();
    setProgress(runtimeOverallPercent(name, 100), INSTALLING_MESSAGE);
    setInstallDetail(`${componentLabel(name)}: 설치 완료`);
    return true;
  } catch (error) {
    installingComponents.delete(name);
    failedComponents.add(name);
    renderComponentStatus(name, { installed: false });
    appendLog(`[오류] ${error}`);
    elements.statusText.textContent = "구성요소 설치 실패";
    setInstallDetail(`${componentLabel(name)}: 설치 실패`);
    return false;
  }
}

async function ensureComponents() {
  if (autoInstallStarted) return;
  autoInstallStarted = true;

  let state;
  try {
    state = await refreshStatus();
  } catch (error) {
    autoInstallStarted = false;
    throw error;
  }

  const missing = missingComponents(state);
  if (!missing.length) return;

  runtimeInstallPlan = missing.map(([name]) => name);
  setBusy(true);
  setProgress(0, INSTALLING_MESSAGE);
  setInstallDetail("필수 구성요소 설치를 준비 중입니다.");
  appendLog("[정보] 필수 구성요소 자동 설치를 시작합니다.");

  try {
    for (const [name] of missing) {
      await installComponent(name);
    }
  } finally {
    setBusy(false);
    const latest = await refreshStatus();
    const stillMissing = missingComponents(latest).length > 0;

    if (stillMissing) {
      autoInstallStarted = false;
      elements.statusText.textContent = "구성요소 설치 확인 필요";
    } else {
      setProgress(100, "다운로드 준비 완료");
      setInstallDetail("필수 파일 설치가 완료되었습니다.");
    }

    runtimeInstallPlan = [];
  }
}

async function startDownload() {
  const url = elements.url.value.trim();
  if (!url) {
    appendLog("[오류] URL을 입력하세요.");
    showPage("download");
    elements.url.focus();
    return;
  }

  if (installingComponents.size) {
    appendLog("[알림] 필수 구성요소 설치가 끝난 뒤 다운로드할 수 있습니다.");
    return;
  }

  showPage("download");
  setBusy(true);
  setProgress(0, "다운로드 준비 중...");
  setInstallDetail("", false);
  resetDownloadMetadata("영상 정보를 불러오는 중");
  elements.log.textContent = "";

  try {
    await saveCurrentSettings(false);
    await invoke("start_download", {
      request: {
        url,
        saveDir: elements.saveDir.value.trim(),
        mode: elements.mode.value,
        quality: elements.quality.value,
        audioQuality: elements.audioQuality.value,
        audioFormat: "original"
      }
    });
  } catch (error) {
    appendLog(`[오류] ${error}`);
    elements.statusText.textContent = "다운로드 실패";
    setBusy(false);
  }
}

async function deleteHistory(id) {
  await invoke("delete_history_record", { id });
  await refreshStatus();
}

async function clearHistory() {
  await invoke("clear_history");
  await refreshStatus();
}

async function chooseDownloadDir() {
  try {
    const selected = await invoke("choose_download_dir", { current: elements.saveDir.value.trim() });
    if (!selected) return;
    elements.saveDir.value = selected;
    await saveCurrentSettings(false);
    appendLog(`[정보] 저장 폴더를 변경했습니다: ${selected}`);
  } catch (error) {
    appendLog(`[오류] 저장 폴더 선택 실패: ${error}`);
  }
}

function wireAdFallbacks() {
  adCards.forEach((card) => {
    const frame = card.querySelector(".ad-frame");
    const fallback = card.querySelector(".ad-fallback");
    if (!frame || !fallback) return;

    let loaded = false;
    frame.addEventListener("load", () => {
      loaded = true;
      fallback.hidden = true;
    });

    setTimeout(() => {
      if (!loaded) fallback.hidden = false;
    }, 5000);
  });
}

function wireEvents() {
  pageButtons.forEach((button) => {
    button.addEventListener("click", () => showPage(button.dataset.pageTarget));
  });

  donationButtons.forEach((button) => {
    button.addEventListener("click", async () => {
      try {
        await invoke("open_external_link", { url: button.dataset.openUrl });
      } catch (error) {
        appendLog(`[오류] 외부 링크 열기 실패: ${error}`);
      }
    });
  });

  elements.refreshStatus.addEventListener("click", async () => {
    try {
      await refreshStatus();
      await ensureComponents();
    } catch (error) {
      appendLog(`[오류] 상태 확인 실패: ${error}`);
    }
  });
  elements.downloadButton.addEventListener("click", startDownload);
  elements.saveSettings.addEventListener("click", () => saveCurrentSettings(true));
  elements.clearHistory.addEventListener("click", clearHistory);
  elements.saveDir.addEventListener("click", chooseDownloadDir);
  elements.saveDir.addEventListener("keydown", (event) => {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    chooseDownloadDir();
  });
  elements.chooseSaveDir.addEventListener("click", chooseDownloadDir);
  elements.acceptLegal.addEventListener("click", async () => {
    elements.acceptLegal.disabled = true;
    try {
      await invoke("accept_legal");
      elements.legalOverlay.hidden = true;
      elements.url.focus();
    } catch (error) {
      appendLog(`[오류] 법적 고지 동의 저장 실패: ${error}`);
    } finally {
      elements.acceptLegal.disabled = false;
    }
  });
}

listen("download-log", (event) => appendLog(event.payload));
listen("download-progress", (event) => {
  setInstallDetail("", false);
  setProgress(event.payload.percent, event.payload.message);
});
listen("download-metadata", (event) => setDownloadMetadata(event.payload));
listen("install-progress", (event) => {
  const { name, percent, message } = event.payload;
  const overall = runtimeOverallPercent(name, percent);
  setProgress(overall, INSTALLING_MESSAGE);
  const label = componentLabel(name);
  const detail = `${label}: ${message || "설치 중"}`;
  setInstallDetail(detail);

  if (percent === 0 || percent === 100 || String(message || "").includes("압축") || String(message || "").includes("검증")) {
    appendLog(`[설치] ${detail}`);
  }
});
listen("download-finished", async (event) => {
  setProgress(100, "다운로드 완료");
  appendLog(event.payload);
  setBusy(false);
  await refreshStatus();
});
listen("download-failed", (event) => {
  elements.statusText.textContent = "다운로드 실패";
  appendLog(`[오류] ${event.payload}`);
  setBusy(false);
});
listen("history-updated", (event) => renderHistory(event.payload));

(async () => {
  try {
    resetDownloadMetadata();
    showPage(activePage);
    wireEvents();
    wireAdFallbacks();
    await refreshStatus();
    await ensureComponents();
  } catch (error) {
    appendLog(`[오류] 초기화 실패: ${error}`);
  }
})();
