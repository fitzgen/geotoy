import { memory } from "./geotoy_wasm_bg";
import {
  create_mesh,
  points_len,
  point_dim,
  points,
  lines_len,
  line_dim,
  lines,
  attractors_len,
  attractor_dim,
  attractors,
  kinds_len,
  kind_dim,
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
  const pointsArray = new Float32Array(memory.buffer, points(), points_len() * point_dim());
  gl.bufferData(gl.ARRAY_BUFFER, pointsArray, gl.STATIC_DRAW);

  attractorsBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, attractorsBuffer);
  const attractorsArray = new Float32Array(memory.buffer, attractors(), attractors_len() * attractor_dim());
  gl.bufferData(gl.ARRAY_BUFFER, attractorsArray, gl.STATIC_DRAW);

  kindsBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, kindsBuffer);
  const kindsArrayInt = new Uint32Array(memory.buffer, kinds(), kinds_len() * kind_dim());
  // GLSL in WebGL does not support integer attributes
  const kindsArrayFloat = new Float32Array(kindsArrayInt);
  gl.bufferData(gl.ARRAY_BUFFER, kindsArrayFloat, gl.STATIC_DRAW);

  linesBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, linesBuffer);
  const linesArray = new Uint16Array(memory.buffer, lines(), lines_len() * line_dim());
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, linesArray, gl.STATIC_DRAW);
  linesBuffer.itemSize = gl.UNSIGNED_SHORT; // u16
  linesBuffer.numItems = lines_len();
}

let animationId = null;
const scheduleDraw = () => {
  if (animationId !== null) {
    return;
  }
  animationId = requestAnimationFrame(() => {
    animationId = null;

    // Properties for arrays; all tightly packed floats
    var type = gl.FLOAT;    // 32bit floating point values
    var normalize = false;  // leave the values as they are
    var offset = 0;         // start at the beginning of the buffer
    var stride = 0;         // how many bytes to move to the next vertex
    // 0 = use the correct stride for type and vertexAttribPointer's size argument (2nd)

    // = Set position attribute =
    gl.bindBuffer(gl.ARRAY_BUFFER, pointsBuffer);
    var positionLoc = gl.getAttribLocation(shaderProgram, "position");
    gl.vertexAttribPointer(positionLoc, point_dim(), type, normalize, stride, offset);
    gl.enableVertexAttribArray(positionLoc);

    // = Set attractor attribute =
    gl.bindBuffer(gl.ARRAY_BUFFER, attractorsBuffer);
    var attractorsLoc = gl.getAttribLocation(shaderProgram, "attractor");
    gl.vertexAttribPointer(attractorsLoc, attractor_dim(), type, normalize, stride, offset);
    gl.enableVertexAttribArray(attractorsLoc);

    // = Set kind attribute =
    gl.bindBuffer(gl.ARRAY_BUFFER, kindsBuffer);
    var kindLoc = gl.getAttribLocation(shaderProgram, "kind");
    gl.vertexAttribPointer(kindLoc, kind_dim(), type, normalize, stride, offset);
    gl.enableVertexAttribArray(kindLoc);

    // = Set uniforms =
    gl.uniform1f(gl.getUniformLocation(shaderProgram, "a"), 0.2);
    gl.uniform1f(gl.getUniformLocation(shaderProgram, "b"), 0.6);

    // = Set index buffer =
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, linesBuffer);

    // = Draw =
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.clearColor(0.0, 0.0, 0.0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);

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
