(() => {
  "use strict";

  const SVG_NS = "http://www.w3.org/2000/svg";
  const STORAGE_KEY = "arrow-zen-clean-v1";

  const DIRS = {
    up: { dx: 0, dy: -1, opposite: "down" },
    right: { dx: 1, dy: 0, opposite: "left" },
    down: { dx: 0, dy: 1, opposite: "up" },
    left: { dx: -1, dy: 0, opposite: "right" }
  };

  const T = {
    e: "empty",
    h: "line-h",
    v: "line-v",
    c: "cross",
    tu: "tee-up",
    tr: "tee-right",
    td: "tee-down",
    tl: "tee-left",
    ur: "corner-ur",
    rd: "corner-rd",
    dl: "corner-dl",
    lu: "corner-lu"
  };

  const LEVELS = [
    {
      name: "Crooked Opening",
      rows: 12,
      cols: 12,
      cells: [
        [T.e, T.h, T.h, T.h, T.rd, T.e, T.e, T.h, T.h, T.h, T.h, T.e],
        [T.e, T.v, T.e, T.e, T.v, T.e, T.e, T.v, T.e, T.e, T.v, T.e],
        [T.e, T.ur, T.h, T.rd, T.c, T.h, T.h, T.c, T.h, T.rd, T.tl, T.e],
        [T.e, T.e, T.e, T.v, T.v, T.e, T.e, T.v, T.e, T.v, T.e, T.e],
        [T.h, T.h, T.h, T.c, T.tu, T.h, T.h, T.c, T.h, T.td, T.h, T.h],
        [T.e, T.e, T.e, T.v, T.e, T.e, T.e, T.v, T.e, T.v, T.e, T.e],
        [T.e, T.e, T.v, T.tl, T.e, T.e, T.h, T.tu, T.h, T.c, T.h, T.h],
        [T.e, T.e, T.v, T.e, T.e, T.e, T.e, T.e, T.e, T.v, T.e, T.e],
        [T.h, T.h, T.c, T.h, T.rd, T.e, T.e, T.h, T.h, T.tu, T.h, T.h],
        [T.e, T.e, T.v, T.e, T.v, T.e, T.e, T.v, T.e, T.e, T.e, T.e],
        [T.e, T.e, T.v, T.h, T.tu, T.h, T.h, T.lu, T.e, T.e, T.e, T.e],
        [T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e]
      ]
    },
    {
      name: "Quiet Switchbacks",
      rows: 13,
      cols: 13,
      cells: [
        [T.e, T.e, T.h, T.h, T.rd, T.e, T.e, T.e, T.h, T.h, T.h, T.h, T.e],
        [T.e, T.e, T.v, T.e, T.v, T.e, T.e, T.e, T.v, T.e, T.e, T.v, T.e],
        [T.h, T.h, T.c, T.h, T.c, T.h, T.rd, T.e, T.ur, T.h, T.h, T.c, T.h],
        [T.e, T.e, T.v, T.e, T.v, T.e, T.v, T.e, T.e, T.e, T.e, T.v, T.e],
        [T.e, T.e, T.ur, T.h, T.tu, T.h, T.c, T.h, T.h, T.rd, T.e, T.v, T.e],
        [T.e, T.e, T.e, T.e, T.e, T.e, T.v, T.e, T.e, T.v, T.e, T.v, T.e],
        [T.h, T.h, T.rd, T.e, T.h, T.h, T.tu, T.h, T.h, T.ur, T.h, T.c, T.h],
        [T.e, T.e, T.v, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.v, T.e],
        [T.e, T.e, T.ur, T.h, T.h, T.rd, T.e, T.h, T.h, T.h, T.rd, T.tl, T.e],
        [T.e, T.e, T.e, T.v, T.e, T.v, T.e, T.v, T.e, T.e, T.v, T.e, T.e],
        [T.e, T.h, T.h, T.c, T.h, T.tu, T.h, T.tu, T.h, T.h, T.lu, T.e, T.e],
        [T.e, T.e, T.e, T.v, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e],
        [T.e, T.e, T.e, T.v, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e]
      ]
    },
    {
      name: "Woven Calm",
      rows: 14,
      cols: 14,
      cells: [
        [T.e, T.h, T.h, T.h, T.rd, T.e, T.e, T.h, T.h, T.rd, T.e, T.e, T.e, T.e],
        [T.e, T.v, T.e, T.e, T.v, T.e, T.e, T.v, T.e, T.v, T.e, T.h, T.h, T.h],
        [T.e, T.ur, T.h, T.rd, T.c, T.h, T.h, T.ur, T.h, T.c, T.h, T.tu, T.h, T.h],
        [T.e, T.e, T.e, T.v, T.v, T.e, T.e, T.e, T.h, T.tu, T.h, T.h, T.rd, T.e],
        [T.h, T.h, T.h, T.c, T.tu, T.h, T.h, T.rd, T.e, T.e, T.e, T.e, T.v, T.e],
        [T.e, T.e, T.e, T.v, T.e, T.e, T.e, T.v, T.e, T.e, T.e, T.e, T.v, T.e],
        [T.e, T.e, T.v, T.tl, T.e, T.e, T.e, T.ur, T.h, T.rd, T.e, T.e, T.v, T.e],
        [T.e, T.e, T.v, T.e, T.e, T.h, T.h, T.h, T.h, T.v, T.e, T.e, T.v, T.e],
        [T.e, T.e, T.ur, T.h, T.rd, T.e, T.e, T.e, T.e, T.ur, T.h, T.rd, T.tl, T.e],
        [T.e, T.e, T.e, T.e, T.v, T.e, T.e, T.e, T.e, T.e, T.e, T.v, T.e, T.e],
        [T.e, T.h, T.h, T.rd, T.tu, T.h, T.h, T.h, T.e, T.h, T.h, T.c, T.h, T.h],
        [T.e, T.e, T.e, T.v, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.v, T.e, T.e],
        [T.e, T.e, T.e, T.v, T.h, T.h, T.h, T.h, T.h, T.h, T.h, T.lu, T.e, T.e],
        [T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e, T.e]
      ]
    }
  ];

  const boardEl = document.getElementById("board");
  const levelLabelEl = document.getElementById("levelLabel");
  const movesLabelEl = document.getElementById("movesLabel");
  const remainingLabelEl = document.getElementById("remainingLabel");
  const clickableLabelEl = document.getElementById("clickableLabel");
  const unlockedLabelEl = document.getElementById("unlockedLabel");
  const bestClearedLabelEl = document.getElementById("bestClearedLabel");
  const lastFinishedLabelEl = document.getElementById("lastFinishedLabel");
  const messageEl = document.getElementById("message");

  const btnRestart = document.getElementById("btnRestart");
  const btnHint = document.getElementById("btnHint");
  const btnNext = document.getElementById("btnNext");

  const state = {
    levelIndex: 0,
    grid: null,
    moves: 0,
    clickableMap: new Map(),
    hintId: null,
    won: false,
    stuck: false,
    progress: loadProgress()
  };

  function loadProgress() {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        return { unlockedLevelCount: 1, bestCleared: 0, lastFinishedLevel: null };
      }
      const parsed = JSON.parse(raw);
      return {
        unlockedLevelCount: Math.max(1, Math.min(LEVELS.length, parsed.unlockedLevelCount || 1)),
        bestCleared: Math.max(0, parsed.bestCleared || 0),
        lastFinishedLevel: parsed.lastFinishedLevel ?? null
      };
    } catch {
      return { unlockedLevelCount: 1, bestCleared: 0, lastFinishedLevel: null };
    }
  }

  function saveProgress() {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state.progress));
  }

  function createCellFromToken(token, x, y) {
    const cell = {
      x,
      y,
      open: { up: false, right: false, down: false, left: false },
      arrowDir: null,
      removed: false
    };

    switch (token) {
      case T.h:
        cell.open.left = true;
        cell.open.right = true;
        break;
      case T.v:
        cell.open.up = true;
        cell.open.down = true;
        break;
      case T.ur:
        cell.open.up = true;
        cell.open.right = true;
        break;
      case T.rd:
        cell.open.right = true;
        cell.open.down = true;
        break;
      case T.dl:
        cell.open.down = true;
        cell.open.left = true;
        break;
      case T.lu:
        cell.open.left = true;
        cell.open.up = true;
        break;
      case T.tu:
        cell.open.left = true;
        cell.open.right = true;
        cell.open.up = true;
        break;
      case T.tr:
        cell.open.up = true;
        cell.open.right = true;
        cell.open.down = true;
        break;
      case T.td:
        cell.open.left = true;
        cell.open.right = true;
        cell.open.down = true;
        break;
      case T.tl:
        cell.open.up = true;
        cell.open.down = true;
        cell.open.left = true;
        break;
      case T.c:
        cell.open.up = true;
        cell.open.right = true;
        cell.open.down = true;
        cell.open.left = true;
        break;
      default:
        break;
    }

    return cell;
  }

  function getCellFromLevel(level, x, y) {
    if (!level || x < 0 || y < 0 || x >= level.cols || y >= level.rows) return null;
    return level.grid[y][x];
  }

  function getCell(x, y) {
    return getCellFromLevel(state.grid, x, y);
  }

  function cellId(cell) {
    return `${cell.x},${cell.y}`;
  }

  function connectedDirs(cell) {
    return ["up", "right", "down", "left"].filter((d) => cell.open[d]);
  }

  function cellDegree(cell) {
    return connectedDirs(cell).length;
  }

  function hasMutualConnection(a, b, dirFromA) {
    return Boolean(a && b && a.open[dirFromA] && b.open[DIRS[dirFromA].opposite]);
  }

  function cellAxis(cell) {
    if (!cell) return null;
    const dirs = connectedDirs(cell);
    if (dirs.length !== 2) return null;
    if (dirs.includes("left") && dirs.includes("right")) return "h";
    if (dirs.includes("up") && dirs.includes("down")) return "v";
    return null;
  }

  function isArrowCell(cell) {
    return Boolean(cell && cell.arrowDir && !cell.removed);
  }

  function isBlockingArrow(cell, originCell) {
    return Boolean(cell && cell !== originCell && cell.arrowDir && !cell.removed);
  }

  function forwardConnected(level, cell, dir) {
    const next = getCellFromLevel(level, cell.x + DIRS[dir].dx, cell.y + DIRS[dir].dy);
    return next && hasMutualConnection(cell, next, dir) ? next : null;
  }

  function stoppingDistanceIgnoringArrows(level, originCell, dir) {
    let current = originCell;
    let steps = 0;
    const safetyMax = level.rows * level.cols * 2;

    while (steps < safetyMax) {
      const next = forwardConnected(level, current, dir);
      if (!next) return steps + 1;
      steps += 1;
      current = next;
      if (cellDegree(current) >= 3) return steps;
      if (!forwardConnected(level, current, dir)) return steps;
    }

    return Infinity;
  }

  function gatherRun(level, startCell, axis, visited) {
    const backward = axis === "h" ? "left" : "up";
    const forward = axis === "h" ? "right" : "down";
    const run = [];
    let current = startCell;

    while (true) {
      const prev = forwardConnected(level, current, backward);
      if (!prev || cellAxis(prev) !== axis) break;
      current = prev;
    }

    while (current) {
      if (cellAxis(current) !== axis) break;
      const key = cellId(current);
      if (!visited.has(key)) {
        visited.add(key);
        run.push(current);
      }
      const next = forwardConnected(level, current, forward);
      if (!next || cellAxis(next) !== axis) break;
      current = next;
    }

    return run;
  }

  function chooseRunDirection(level, run, runIndex) {
    const axis = cellAxis(run[0]);
    const negative = axis === "h" ? "left" : "up";
    const positive = axis === "h" ? "right" : "down";

    const totalNegative = run.reduce((sum, cell) => sum + stoppingDistanceIgnoringArrows(level, cell, negative), 0);
    const totalPositive = run.reduce((sum, cell) => sum + stoppingDistanceIgnoringArrows(level, cell, positive), 0);

    if (totalNegative < totalPositive) return negative;
    if (totalPositive < totalNegative) return positive;
    return (runIndex + level.rows + level.cols) % 2 === 0 ? positive : negative;
  }

  function frontmostCellForDirection(run, dir) {
    return dir === "right" || dir === "down" ? run[run.length - 1] : run[0];
  }

  function populateArrowsFromRuns(level) {
    const visited = new Set();
    const runs = [];

    for (let y = 0; y < level.rows; y++) {
      for (let x = 0; x < level.cols; x++) {
        const cell = getCellFromLevel(level, x, y);
        const axis = cellAxis(cell);
        if (!axis || visited.has(cellId(cell))) continue;
        const run = gatherRun(level, cell, axis, visited);
        if (run.length >= 2) runs.push(run);
      }
    }

    runs.forEach((run, index) => {
      const dir = chooseRunDirection(level, run, index);
      run.forEach((cell) => {
        cell.arrowDir = dir;
        cell.removed = false;
      });

      if (index < 3) level.tutorialArrows.add(cellId(frontmostCellForDirection(run, dir)));
    });
  }

  function buildLevel(levelIndex) {
    const src = LEVELS[levelIndex];
    const grid = Array.from({ length: src.rows }, (_, y) =>
      Array.from({ length: src.cols }, (_, x) => createCellFromToken(src.cells[y][x], x, y))
    );

    const level = {
      name: src.name,
      rows: src.rows,
      cols: src.cols,
      tutorialArrows: new Set(),
      grid
    };

    populateArrowsFromRuns(level);
    return level;
  }

  function countRemainingArrows() {
    let total = 0;
    for (let y = 0; y < state.grid.rows; y++) {
      for (let x = 0; x < state.grid.cols; x++) {
        if (isArrowCell(getCell(x, y))) total += 1;
      }
    }
    return total;
  }

  function traceForwardOutcome(originCell) {
    if (!isArrowCell(originCell)) return { clickable: false, reason: "not-arrow" };

    const dir = originCell.arrowDir;
    const next = getCell(originCell.x + DIRS[dir].dx, originCell.y + DIRS[dir].dy);

    if (!next || !hasMutualConnection(originCell, next, dir)) {
      return { clickable: false, reason: "blocked-immediately" };
    }
    if (isBlockingArrow(next, originCell)) {
      return { clickable: false, reason: "arrow-blocked-immediately" };
    }

    let current = next;
    let steps = 1;
    const safetyMax = state.grid.rows * state.grid.cols * 2;

    while (steps <= safetyMax) {
      if (isBlockingArrow(current, originCell)) return { clickable: false, reason: "hit-unresolved-arrow" };
      if (cellDegree(current) >= 3) return { clickable: true, reason: "junction" };

      const nextStraight = getCell(current.x + DIRS[dir].dx, current.y + DIRS[dir].dy);
      if (!nextStraight || !hasMutualConnection(current, nextStraight, dir)) {
        return { clickable: true, reason: "stopping-point" };
      }
      if (isBlockingArrow(nextStraight, originCell)) {
        return { clickable: false, reason: "hit-unresolved-arrow" };
      }

      current = nextStraight;
      steps += 1;
    }

    return { clickable: false, reason: "safety-stop" };
  }

  function isClickableArrow(cell) {
    return traceForwardOutcome(cell).clickable;
  }

  function onLevelCleared() {
    const clearedNumber = state.levelIndex + 1;

    if (clearedNumber > state.progress.bestCleared) state.progress.bestCleared = clearedNumber;
    state.progress.lastFinishedLevel = clearedNumber;

    if (state.progress.unlockedLevelCount < LEVELS.length) {
      state.progress.unlockedLevelCount = Math.max(
        state.progress.unlockedLevelCount,
        Math.min(LEVELS.length, state.levelIndex + 2)
      );
    }

    saveProgress();
  }

  function computeClickable() {
    const map = new Map();

    for (let y = 0; y < state.grid.rows; y++) {
      for (let x = 0; x < state.grid.cols; x++) {
        const cell = getCell(x, y);
        if (isClickableArrow(cell)) map.set(cellId(cell), true);
      }
    }

    state.clickableMap = map;

    const remaining = countRemainingArrows();
    const clickable = map.size;
    state.won = remaining === 0;
    state.stuck = remaining > 0 && clickable === 0;

    if (state.won) onLevelCleared();
  }

  function setMessage(text) {
    messageEl.textContent = text;
  }

  function updateStatus() {
    levelLabelEl.textContent = `${state.levelIndex + 1} / ${LEVELS.length}`;
    movesLabelEl.textContent = String(state.moves);
    remainingLabelEl.textContent = String(countRemainingArrows());
    clickableLabelEl.textContent = String(state.clickableMap.size);
    unlockedLabelEl.textContent = String(state.progress.unlockedLevelCount);
    bestClearedLabelEl.textContent = String(state.progress.bestCleared);
    lastFinishedLabelEl.textContent = state.progress.lastFinishedLevel ? `Level ${state.progress.lastFinishedLevel}` : "None";

    if (state.won) setMessage("Beautiful. You cleared the maze.");
    else if (state.stuck) setMessage("No valid arrows remain. Restart or try the next level.");
  }

  function finalizeArrowRemoval(cell) {
    cell.removed = true;
    cell.arrowDir = null;
    state.moves += 1;
    state.hintId = null;

    computeClickable();
    updateStatus();
    render();

    if (!state.won && !state.stuck) setMessage("Nice. That opened the maze a bit more.");
  }

  function animateArrowRemoval(cell) {
    const id = cellId(cell);
    const group = boardEl.querySelector(`[data-arrow-id="${cssEscape(id)}"]`);

    if (!group) {
      finalizeArrowRemoval(cell);
      return;
    }

    group.classList.add("removing");

    const bboxSource = group.querySelector(".arrow-line");
    if (bboxSource) {
      const box = bboxSource.getBBox();
      const spark = svgEl("circle", {
        cx: box.x + box.width / 2,
        cy: box.y + box.height / 2,
        r: 5,
        class: "spark"
      });
      boardEl.appendChild(spark);
      setTimeout(() => spark.remove(), 360);
    }

    setTimeout(() => finalizeArrowRemoval(cell), 250);
  }

  function removeArrow(cell) {
    if (!isArrowCell(cell)) return;

    const id = cellId(cell);
    if (!state.clickableMap.has(id)) {
      const outcome = traceForwardOutcome(cell);
      if (outcome.reason === "arrow-blocked-immediately" || outcome.reason === "hit-unresolved-arrow") {
        setMessage("That arrow is still blocked by another unresolved arrow ahead.");
      } else if (outcome.reason === "blocked-immediately") {
        setMessage("That arrow does not yet point into an open straight path.");
      } else {
        setMessage("That arrow does not yet have a valid straight-ahead exit.");
      }
      return;
    }

    animateArrowRemoval(cell);
  }

  function showHint() {
    const ids = [...state.clickableMap.keys()];
    if (!ids.length) {
      state.hintId = null;
      render();
      setMessage("No valid hint is available.");
      return;
    }

    state.hintId = ids[Math.floor(Math.random() * ids.length)];
    render();
    setMessage("Hint: the highlighted red arrow has a valid route.");
  }

  function resetLevel(levelIndex = state.levelIndex) {
    state.levelIndex = levelIndex;
    state.grid = buildLevel(levelIndex);
    state.moves = 0;
    state.clickableMap = new Map();
    state.hintId = null;
    state.won = false;
    state.stuck = false;

    computeClickable();
    updateStatus();
    render();
    setMessage(`Level ${levelIndex + 1}: ${state.grid.name}`);
  }

  function nextLevel() {
    const maxUnlocked = state.progress.unlockedLevelCount;
    const next = state.levelIndex + 1;

    if (next < maxUnlocked && next < LEVELS.length) {
      resetLevel(next);
      return;
    }

    if (state.levelIndex + 1 >= LEVELS.length) {
      resetLevel(0);
      return;
    }

    setMessage("Clear the current frontier to unlock the next level.");
  }

  function svgEl(tag, attrs) {
    const node = document.createElementNS(SVG_NS, tag);
    Object.entries(attrs).forEach(([k, v]) => node.setAttribute(k, String(v)));
    return node;
  }

  function line(x1, y1, x2, y2, className) {
    return svgEl("line", { x1, y1, x2, y2, class: className });
  }

  function arrowPath(dir, cx, cy, len) {
    const shaft = len * 0.42;
    const head = len * 0.16;

    switch (dir) {
      case "up":
        return [`M ${cx} ${cy + shaft}`, `L ${cx} ${cy - shaft}`, `M ${cx} ${cy - shaft}`, `L ${cx - head} ${cy - shaft + head}`, `M ${cx} ${cy - shaft}`, `L ${cx + head} ${cy - shaft + head}`].join(" ");
      case "right":
        return [`M ${cx - shaft} ${cy}`, `L ${cx + shaft} ${cy}`, `M ${cx + shaft} ${cy}`, `L ${cx + shaft - head} ${cy - head}`, `M ${cx + shaft} ${cy}`, `L ${cx + shaft - head} ${cy + head}`].join(" ");
      case "down":
        return [`M ${cx} ${cy - shaft}`, `L ${cx} ${cy + shaft}`, `M ${cx} ${cy + shaft}`, `L ${cx - head} ${cy + shaft - head}`, `M ${cx} ${cy + shaft}`, `L ${cx + head} ${cy + shaft - head}`].join(" ");
      case "left":
        return [`M ${cx + shaft} ${cy}`, `L ${cx - shaft} ${cy}`, `M ${cx - shaft} ${cy}`, `L ${cx - shaft + head} ${cy - head}`, `M ${cx - shaft} ${cy}`, `L ${cx - shaft + head} ${cy + head}`].join(" ");
      default:
        return "";
    }
  }

  function drawOverlay(root, width, height, textValue) {
    const overlay = svgEl("g", { class: "overlay" });

    overlay.appendChild(svgEl("rect", {
      x: width * 0.25,
      y: height * 0.43,
      width: width * 0.5,
      height: 70,
      rx: 18
    }));

    const text = svgEl("text", {
      x: width / 2,
      y: height * 0.43 + 43,
      "text-anchor": "middle",
      "font-size": "30"
    });
    text.textContent = textValue;

    overlay.appendChild(text);
    root.appendChild(overlay);
  }

  function drawCell(root, cell, size, pad) {
    const x0 = pad + cell.x * size;
    const y0 = pad + cell.y * size;
    const cx = x0 + size / 2;
    const cy = y0 + size / 2;
    const arm = size * 0.48;

    if (cell.open.up) root.appendChild(line(cx, cy, cx, cy - arm, "corridor"));
    if (cell.open.right) root.appendChild(line(cx, cy, cx + arm, cy, "corridor"));
    if (cell.open.down) root.appendChild(line(cx, cy, cx, cy + arm, "corridor"));
    if (cell.open.left) root.appendChild(line(cx, cy, cx - arm, cy, "corridor"));

    if (cell.arrowDir && !cell.removed) {
      const id = cellId(cell);
      const clickable = state.clickableMap.has(id);
      const hint = state.hintId === id;
      const tutorial = state.grid.tutorialArrows.has(id);

      const group = svgEl("g", { class: "arrow-group", "data-arrow-id": id });
      const d = arrowPath(cell.arrowDir, cx, cy, size * 0.63);

      const hit = svgEl("path", { d, class: "arrow-hit" });
      const classes = ["arrow-line"];
      if (clickable) classes.push("clickable");
      if (tutorial) classes.push("tutorial");
      if (hint) classes.push("hint");

      const visible = svgEl("path", { d, class: classes.join(" ") });
      hit.addEventListener("click", () => removeArrow(cell));

      group.appendChild(hit);
      group.appendChild(visible);
      root.appendChild(group);
    }
  }

  function render() {
    const rows = state.grid.rows;
    const cols = state.grid.cols;
    const size = 48;
    const pad = 20;
    const width = cols * size + pad * 2;
    const height = rows * size + pad * 2;

    boardEl.setAttribute("viewBox", `0 0 ${width} ${height}`);
    boardEl.innerHTML = "";

    const root = svgEl("g", {});
    boardEl.appendChild(root);

    for (let y = 0; y < rows; y++) {
      for (let x = 0; x < cols; x++) drawCell(root, getCell(x, y), size, pad);
    }

    if (state.won || state.stuck) drawOverlay(root, width, height, state.won ? "Level Cleared" : "No Moves");
  }

  function cssEscape(value) {
    if (window.CSS && typeof window.CSS.escape === "function") return window.CSS.escape(value);
    return value.replace(/([ #;?%&,.+*~':"!^$[\]()=>|/@])/g, "\\$1");
  }

  btnRestart.addEventListener("click", () => resetLevel(state.levelIndex));
  btnHint.addEventListener("click", showHint);
  btnNext.addEventListener("click", nextLevel);

  resetLevel(0);
})();
