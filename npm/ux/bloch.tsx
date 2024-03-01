// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

/* TODO:

- Show the state vector / linear combination
- Add the trailing dots with a slider for history and fade out speed
- Animiate the axes (including for H) when rotating
  - Maybe add a new geo/mesh for axis with rotation arrow that rotates with the qubit
- Have the marker indicate the phase/direction
- Show the matrix to be applied when hovering over a gate

*/

import { useEffect, useRef, useState } from "preact/hooks";

import {
  ConeGeometry,
  CylinderGeometry,
  DirectionalLight,
  Group,
  LineSegments,
  Mesh,
  MeshBasicMaterial,
  MeshBasicMaterialParameters,
  MeshLambertMaterial,
  PerspectiveCamera,
  Quaternion,
  Scene,
  SphereGeometry,
  Vector3,
  WebGLRenderer,
  WireframeGeometry,
} from "three";

import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

import { FontLoader } from "three/examples/jsm/loaders/FontLoader.js";
import { TextGeometry } from "three/examples/jsm/geometries/TextGeometry.js";

const colors = {
  sphereColor: 0x404080,
  sphereBrightness: 2,
  sphereOpacity: 0.5,
  directionalLightBrightness: 0.25,
  markerColor: 0xc00000,
  sphereLinesOpacity: 0.2,
};

const fontMap = {
  helvetiker: 0,
  optimer: 1,
  gentilis: 2,
  "droid/droid_sans": 3,
  "droid/droid_serif": 4,
};

const weightMap = {
  regular: 0,
  bold: 1,
};

// See https://gizma.com/easing/#easeInOutSine
function easeInOutSine(x: number) {
  return -(Math.cos(Math.PI * x) - 1) / 2;
}

function createText(scene: Scene, done: () => void) {
  const loader = new FontLoader();
  const fontMat = new MeshBasicMaterial({ color: 0x606080, opacity: 1 });
  loader.load("fonts/helvetiker_regular.typeface.json", (font) => {
    const fontProps = {
      font,
      size: 0.6,
      height: 0.01,
      bevelThickness: 0.075,
      bevelSize: 0.01,
      bevelEnabled: true,
    };
    const xGeo = new TextGeometry("x", fontProps);
    const yGeo = new TextGeometry("y", fontProps);
    const zGeo = new TextGeometry("z", fontProps);

    const xMesh = new Mesh(xGeo, fontMat);
    const yMesh = new Mesh(yGeo, fontMat);
    const zMesh = new Mesh(zGeo, fontMat);
    xGeo.computeBoundingBox();
    yGeo.computeBoundingBox();
    zGeo.computeBoundingBox();

    xMesh.position.set(
      -0.5 * (xGeo.boundingBox!.max.x - xGeo.boundingBox!.min.x),
      -0.5 * (xGeo.boundingBox!.max.y - xGeo.boundingBox!.min.y),
      6.4,
    );
    yMesh.position.set(
      6.4,
      -0.5 * (xGeo.boundingBox!.max.y - xGeo.boundingBox!.min.y),
      0,
    );

    zMesh.position.set(
      -0.5 * (zGeo.boundingBox!.max.x - zGeo.boundingBox!.min.x),
      6.4,
      0,
    );
    scene.add(xMesh);
    scene.add(yMesh);
    scene.add(zMesh);
    done();
  });
}

const rotationTimeMs = 750;

class BlochRenderer {
  scene: Scene;
  camera: PerspectiveCamera;
  renderer: WebGLRenderer;
  controls: OrbitControls;
  qubit: Group;

  constructor(canvas: HTMLCanvasElement) {
    // For VS Code, WebView body attribute 'data-vscode-theme-kind' will contain 'light' if light theme is active.
    // Note: The value is usually 'vscode-light' or 'vscode-dark', but high-contrast dark is just 'vscode-high-contrast',
    // whereas the light high contract theme is 'vscode-high-contrast-light'.
    // Default to 'light' if the attribute is not present. (e.g. in the Playground)
    const isLight = (
      document.body.getAttribute("data-vscode-theme-kind") ?? "light"
    ).includes("light");

    const renderer = new WebGLRenderer({
      canvas,
      antialias: true,
      alpha: true,
    });

    const scene = new Scene();
    const camera = new PerspectiveCamera(
      30, // fov
      1, // aspect
      0.1, // near
      1000, // far
    );

    // In WebGL, Z is towards the camera (viewer looking towards -Z), Y is up, X is right
    // Position slightly towards the X and Y axis to give a 'canonical' view
    camera.position.x = 4;
    camera.position.y = 4;
    camera.position.z = 27;
    camera.lookAt(0, 0, 0);

    const light = new DirectionalLight(
      0xffffff,
      colors.directionalLightBrightness,
    );
    light.position.set(-1, 2, 4);
    scene.add(light);

    // Note that the orbit controls move the camera, they don't rotate the
    // scene, so the X, Y, and Z axis for the Bloch sphere remain fixed.
    const controls = new OrbitControls(camera, renderer.domElement);

    // Create a group to hold the qubit
    const qubit = new Group();
    // Add the main sphere
    const sphereGeometry = new SphereGeometry(5, 32, 16);
    const material = new MeshLambertMaterial({
      emissive: colors.sphereColor,
      emissiveIntensity: colors.sphereBrightness,
      transparent: true,
      opacity: colors.sphereOpacity,
    });
    const sphere = new Mesh(sphereGeometry, material);
    qubit.add(sphere);

    // Add the 'spin' direction marker
    const coneGeometry = new ConeGeometry(0.2, 0.75, 32);
    const coneMat = new MeshBasicMaterial({ color: colors.markerColor });
    const marker = new Mesh(coneGeometry, coneMat);
    marker.position.set(0, 5.125, 0.4);
    marker.rotateX(Math.PI / 2);
    qubit.add(marker);

    // Draw the wires on it
    const sphereWireGeometry = new SphereGeometry(5.1, 16, 16);
    const wireframe = new WireframeGeometry(sphereWireGeometry);
    const sphereLines = new LineSegments(wireframe);
    const materialProps = sphereLines.material as MeshBasicMaterialParameters;
    materialProps.depthTest = true;
    materialProps.opacity = colors.sphereLinesOpacity;
    materialProps.transparent = true;
    qubit.add(sphereLines);
    scene.add(qubit);

    // Add the axes
    const axisMaterial = new MeshBasicMaterial({ color: 0xe0d0c0 });
    const zAxis = new CylinderGeometry(0.075, 0.075, 12, 32, 8);
    const zAxisMesh = new Mesh(zAxis, axisMaterial);
    scene.add(zAxisMesh);

    const zPointer = new ConeGeometry(0.2, 0.8, 16);
    const zPointerMesh = new Mesh(zPointer, axisMaterial);
    zPointerMesh.position.set(0, 6, 0);
    scene.add(zPointerMesh);

    const yAxisMesh = new Mesh(zAxis, axisMaterial);
    yAxisMesh.rotateZ(Math.PI / 2);
    scene.add(yAxisMesh);
    const yPointerMesh = new Mesh(zPointer, axisMaterial);
    yPointerMesh.position.set(6, 0, 0);
    yPointerMesh.rotateZ(-Math.PI / 2);
    scene.add(yPointerMesh);

    const xAxisMesh = new Mesh(zAxis, axisMaterial);
    xAxisMesh.rotateX(Math.PI / 2);
    scene.add(xAxisMesh);
    const xPointerMesh = new Mesh(zPointer, axisMaterial);
    xPointerMesh.position.set(0, 0, 6);
    xPointerMesh.rotateX(Math.PI / 2);
    scene.add(xPointerMesh);

    // See https://threejs.org/manual/#en/rendering-on-demand
    controls.addEventListener("change", () =>
      requestAnimationFrame(() => this.render()),
    );

    this.renderer = renderer;
    this.scene = scene;
    this.camera = camera;
    this.controls = controls;
    this.qubit = qubit;

    // Initial render
    //requestAnimationFrame(() => this.render());
    createText(scene, () => this.render());
  }

  rotate(axis: Vector3, angle: number) {
    const startTimeMs = performance.now();

    const initial = this.qubit.quaternion.clone();
    const target = new Quaternion()
      .setFromAxisAngle(axis, angle)
      .multiply(initial);

    // Also slerp the regular intervals between the two quaternions and
    // translate to world space
    // TODO:
    // - Add only as the path is travelled
    // - Fade out old ones per the current settings

    /*
To calculate the distance the point travels as the rotation is applied.
- Calculate the angle (theta) between the axis of rotation and the point
- Get the circumfence for the circle around the sphere at latitude theta: sin(theta) * 2 * PI
- Calculate the distance travelled as the angle * circumfence
    */

    const qubitPointInLocal = new Vector3(0, 5, 0);
    const qubitPointInWorld = qubitPointInLocal
      .clone()
      .applyQuaternion(initial);
    const qubitToAxisAngle = qubitPointInWorld.angleTo(axis);
    const pathTravelled = Math.sin(qubitToAxisAngle) * angle;
    // console.log(
    //   `Path travelled: ${pathTravelled}, angle from axis: ${qubitToAxisAngle}`,
    // );

    for (let i = 0; i < 1; i += 1 / (pathTravelled * 25)) {
      const q = initial.clone().slerp(target, i);
      // Conver to world space
      const trackGeo = new SphereGeometry(0.05, 16, 16);
      const trackBall = new Mesh(
        trackGeo,
        new MeshBasicMaterial({ color: 0x808080 }),
      );
      trackBall.position.set(0, 5, 0);
      trackBall.position.applyQuaternion(q);
      this.scene.add(trackBall);
    }

    const update = () => {
      const now = performance.now();

      const x = Math.max(now - startTimeMs, 1) / rotationTimeMs;
      const t = x < 1 ? easeInOutSine(x) : 1;
      this.qubit.quaternion.slerpQuaternions(initial, target, t);
      this.render();
      if (t < 1) {
        requestAnimationFrame(update);
      }
    };
    requestAnimationFrame(update);
  }

  // The Bloch sphere X axis is the Z axis in WebGL
  rotateX(angle: number) {
    this.rotate(new Vector3(0, 0, 1), angle);
  }
  // The Bloch sphere Y axis is the X axis in WebGL
  rotateY(angle: number) {
    this.rotate(new Vector3(1, 0, 0), angle);
  }

  // The Bloch sphere Z axis is the Y axis in WebGL
  rotateZ(angle: number) {
    this.rotate(new Vector3(0, 1, 0), angle);
  }

  rotateH(angle: number) {
    const hAxis = new Vector3(0, 1, 1).normalize();
    this.rotate(hAxis, angle);
  }

  reset() {
    this.qubit.rotation.set(0, 0, 0);
    this.render();
  }

  render() {
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }
}

export function BlochSphere(props: { gates?: string[] }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const renderer = useRef<BlochRenderer | null>(null);

  const [gates, setGates] = useState<string[]>(props.gates || []);

  useEffect(() => {
    if (canvasRef.current) {
      renderer.current = new BlochRenderer(canvasRef.current);
    }
  }, []);

  function rotate(gate: string): void {
    setGates([...gates, gate]);
    if (renderer.current) {
      switch (gate) {
        case "X":
          renderer.current.rotateX(Math.PI);
          break;
        case "Y":
          renderer.current.rotateY(Math.PI);
          break;
        case "Z":
          renderer.current.rotateZ(Math.PI);
          break;
        case "S":
          renderer.current.rotateZ(Math.PI / 2);
          break;
        case "T":
          renderer.current.rotateZ(Math.PI / 4);
          break;
        case "H":
          renderer.current.rotateH(Math.PI);
          break;
        default:
          console.error("Unknown gate: " + gate);
      }
    }
  }

  function reset() {
    setGates([]);
    if (renderer.current) {
      renderer.current.reset();
    }
  }

  return (
    <div style="position: relative;">
      <canvas ref={canvasRef} width="600" height="600"></canvas>
      <div>{"Applied: " + gates.join(", ")}</div>
      <div class="qs-gate-buttons">
        <button type="button" onClick={() => rotate("X")}>
          X
        </button>
        <button type="button" onClick={() => rotate("Y")}>
          Y
        </button>
        <button type="button" onClick={() => rotate("Z")}>
          Z
        </button>
        <button type="button" onClick={() => rotate("S")}>
          S
        </button>
        <button type="button" onClick={() => rotate("T")}>
          T
        </button>
        <button type="button" onClick={() => rotate("H")}>
          H
        </button>
        <button type="button" onClick={reset}>
          Reset
        </button>
      </div>
    </div>
  );
}
