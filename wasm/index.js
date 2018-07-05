import { memory } from "./geotoy_wasm_bg";
import {
  create_mesh,
  points_len,
  size_of_point,
  points,
  lines_len,
  size_of_line,
  lines,
  attractors_len,
  size_of_attractor,
  attractors,
  kinds_len,
  size_of_kind,
  kinds,
  vertex_shader,
  fragment_shader
} from "./geotoy_wasm";

const canvas = window.canvas = document.getElementById("canvas");
const gl = window.gl = canvas.getContext("webgl");

let size = 0;
let rows = 0;
let columns = 0;

const onResize = () => {
  //size = Math.ceil(Math.min(canvas.width, canvas.height) / 5);
  //rows = Math.ceil(canvas.height / size);
  //columns = Math.ceil(canvas.width / size);

  rows = 5;
  columns = 5;
  size = (1.0 - -1.0) / ((columns - 1) * 1.5);

  createMesh();
  scheduleDraw();
};
window.addEventListener("resize", onResize);

let pointsBuffer = null;
let attractorsBuffer = null;
let kindsBuffer = null;
let linesBuffer = null;
let shaderProgram = null;
const createMesh = () => {
  create_mesh(rows, columns, size);

  pointsBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, pointsBuffer);
  const pointsArray = new Float32Array(memory.buffer, points(), points_len());
  gl.bufferData(gl.ARRAY_BUFFER, pointsArray, gl.STATIC_DRAW);
  pointsBuffer.itemSize = size_of_point();
  pointsBuffer.numItems = points_len();

  attractorsBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, attractorsBuffer);
  const attractorsArray = new Float32Array(memory.buffer, attractors(), attractors_len());
  gl.bufferData(gl.ARRAY_BUFFER, attractorsArray, gl.STATIC_DRAW);
  attractorsArray.itemSize = size_of_attractor();
  attractorsArray.numItems = attractors_len();

  kindsBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, kindsBuffer);
  const kindsArray = new Uint32Array(memory.buffer, kinds(), kinds_len());
  gl.bufferData(gl.ARRAY_BUFFER, kindsArray, gl.STATIC_DRAW);
  kindsArray.itemSize = size_of_kind();
  kindsArray.numItems = kinds_len();

  linesBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, linesBuffer);
  const linesArray = new Uint16Array(memory.buffer, lines(), lines_len() / 3);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, linesArray, gl.STATIC_DRAW);
  linesBuffer.itemSize = gl.UNSIGNED_SHORT; // u16
  linesBuffer.numItems = lines_len() / 3;
}

let animationId = null;
const scheduleDraw = () => {
  if (animationId !== null) {
    return;
  }
  animationId = requestAnimationFrame(() => {
    animationId = null;

    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.clearColor(0.0, 0.0, 0.0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

    gl.bindBuffer(gl.ARRAY_BUFFER, pointsBuffer);
    var positionLoc = gl.getAttribLocation(shaderProgram, "position");

    var numComponents = 2;  // (x, y)
    var type = gl.FLOAT;    // 32bit floating point values
    var normalize = false;  // leave the values as they are
    var offset = 0;         // start at the beginning of the buffer
    var stride = 0;         // how many bytes to move to the next vertex
    // 0 = use the correct stride for type and numComponents

    gl.vertexAttribPointer(positionLoc, numComponents, type, normalize, stride, offset);
    gl.enableVertexAttribArray(positionLoc);

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, linesBuffer);

    gl.drawElements(gl.LINES, linesBuffer.numItems, linesBuffer.itemSize, 0);
  });
};

const compileShaders = () => {
  shaderProgram = gl.createProgram();

  const vertexShader = gl.createShader(gl.VERTEX_SHADER);
  gl.shaderSource(vertexShader, vertex_shader());
  gl.compileShader(vertexShader);
  if (!gl.getShaderParameter(vertexShader, gl.COMPILE_STATUS)) {
    throw new Error("could not compile vertex shader: " + gl.getShaderInfoLog(vertexShader));
  }
  gl.attachShader(shaderProgram, vertexShader);

  const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);
  gl.shaderSource(fragmentShader, fragment_shader());
  gl.compileShader(fragmentShader);
  if (!gl.getShaderParameter(fragmentShader, gl.COMPILE_STATUS)) {
    throw new Error("could not compile fragment shader: " + gl.getShaderInfoLog(fragmentShader));
  }
  gl.attachShader(shaderProgram, fragmentShader);

  gl.linkProgram(shaderProgram);

  if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
    throw new Error("Could not link shaders");
  }

  gl.useProgram(shaderProgram);
};

const main = () => {
  compileShaders()
  onResize();
  scheduleDraw();
};

main();
