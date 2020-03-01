import { Universe, Cell } from "wasm-game-of-life";
import { memory } from "wasm-game-of-life/wasm_game_of_life_bg";

const CELL_SIZE =  5; // pixels
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

// Construct the universe, and get its width and height.
let universe = Universe.new();
const width = universe.width();
const height = universe.height();

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');

let animationId = null;

const isPaused = () => {
    return animationId === null;
};

const playPauseButton = document.getElementById("play-pause");

const play = () => {
    playPauseButton.textContent = "⏸";
    renderLoop();
};

const pause = () => {
    playPauseButton.textContent = "▶";
    cancelAnimationFrame(animationId);
    animationId = null;
};

playPauseButton.addEventListener("click", event => {
    if (isPaused()) {
        play();
    } else {
        pause();
    }
});

// This block of code handles setting default values for ticks per frame
// and wiring up the events for the input range UI elements.
const ticksFrameInputRange = document.getElementById("ticks-frame");
const ticksValue = document.getElementById('ticks-value');
let ticksPerFrame = 1;

const updateTicks = () => {
    ticksPerFrame = ticksFrameInputRange.value;
    ticksValue.innerHTML = ticksFrameInputRange.value;
};

ticksFrameInputRange.addEventListener('input', updateTicks);

// Resets the universe to a random initial state when that button is pressed.
const resetButton = document.getElementById("reset");

resetButton.addEventListener('click', event => {
    universe = Universe.random_universe();
    drawGrid();
    drawCells();
});

// Kills the current universe
const killButton = document.getElementById("all-dead");

killButton.addEventListener('click', event => {
    pause();
    universe.kill_universe();
    drawGrid();
    drawCells();
});

// Allows for click on a grid cell to toggle the state of that cell.
canvas.addEventListener("click", event => {
    const boundingRect = canvas.getBoundingClientRect();

    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (event.clientX - boundingRect.left) * scaleX;
    const canvasTop = (event.clientY - boundingRect.top) * scaleY;

    const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
    const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);

    universe.toggle_cell(row, col);

    drawGrid();
    drawCells();
});

const renderLoop = () => {
    // debugger; // Allows for stepping through the code.
    
    // Uses the input from the range to tick that number of generations before
    // updating the grid & cells.
    for (let i = 0; i < ticksPerFrame; i++) {
        universe.tick();
    }
    drawGrid();
    drawCells();

    animationId = requestAnimationFrame(renderLoop);
};

const drawGrid = () => {
    ctx.beginPath();
    ctx.strokeStyle = GRID_COLOR;

    // Vertical lines.
    for (let i = 0; i <= width; i++) {
        ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
        ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
    }

    // Horizontal lines.
    for (let j = 0; j <= height; j++) {
        ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
        ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
    }

    ctx.stroke();
};

const getIndex = (row, column) => {
    return row * width + column;
};

const drawCells = () => {
    // universe.cells returns a pointer to a memory location
    const cellsPtr = universe.cells();
    // WASM can access Rust's linear memory, cellsPts points to the location in memory
    const cells = new Uint8Array(memory.buffer, cellsPtr, width * height);

    ctx.beginPath();

    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col ++) {
            const idx = getIndex(row, col);

            ctx.fillStyle = cells[idx] === Cell.Dead
                ? DEAD_COLOR
                : ALIVE_COLOR;
            
            ctx.fillRect(
                col * (CELL_SIZE + 1) + 1,
                row * (CELL_SIZE + 1) + 1,
                CELL_SIZE,
                CELL_SIZE
            );
        }
    }

    ctx.stroke();
};

updateTicks();
drawGrid();
drawCells();
play();