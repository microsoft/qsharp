// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* eslint-disable @typescript-eslint/no-unused-vars */

/* TODO:

- Add the labels to the axes
- Add the trailing dots with a slider for history and fade out speed
- Animiate the axes when rotating
- Have the marker indicate the phase/direction
- Move to the npm package as a control

*/

import { useEffect, useRef, useState } from "preact/hooks";

import {
  BoxGeometry,
  ConeGeometry,
  CylinderGeometry,
  DirectionalLight,
  EllipseCurve,
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

// See https://gizma.com/easing/#easeInOutSine
function easeInOutSine(x: number) {
  return -(Math.cos(Math.PI * x) - 1) / 2;
}

const rotationTimeMs = 1000;

class BlochRenderer {
  scene: Scene;
  camera: PerspectiveCamera;
  renderer: WebGLRenderer;
  controls: OrbitControls;
  qubit: Group;

  constructor(canvas: HTMLCanvasElement) {
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

    const light = new DirectionalLight(0xffffff, 0.5);
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
      emissive: 0x404080,
      emissiveIntensity: 3,
      transparent: true,
      opacity: 0.5,
    });
    const sphere = new Mesh(sphereGeometry, material);
    qubit.add(sphere);

    // Add the 'spin' direction marker
    const boxGeo = new BoxGeometry(0.5, 0.25, 0.5);
    const boxMat = new MeshBasicMaterial({ color: 0xff0000 });
    const box = new Mesh(boxGeo, boxMat);
    box.position.set(0, 5.125, 0);
    qubit.add(box);

    // Draw the wires on it
    const sphereWireGeometry = new SphereGeometry(5, 16, 16);
    const wireframe = new WireframeGeometry(sphereWireGeometry);
    const sphereLines = new LineSegments(wireframe);
    const materialProps = sphereLines.material as MeshBasicMaterialParameters;
    materialProps.depthTest = false;
    materialProps.opacity = 0.1;
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

    // // Add the points from |0> to |1>
    // const curve = new EllipseCurve(0, 0, 5.05, 5.05, 0, Math.PI);
    // const points = curve.getPoints(32);
    // const pointMaterial = new MeshBasicMaterial({ color: 0x80ffd0 });

    // const pointsGroup = new Group();
    // for (const point of points) {
    //   const pointGeometry = new SphereGeometry(0.1, 16, 16);
    //   const pointMesh = new Mesh(pointGeometry, pointMaterial);
    //   pointMesh.position.set(0, point.x, point.y);
    //   pointsGroup.add(pointMesh);
    // }
    // pointsGroup.rotateY(0.5);
    // scene.add(pointsGroup);

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
    requestAnimationFrame(() => this.render());
  }

  rotate(axis: Vector3, angle: number) {
    const startTimeMs = performance.now();

    const initial = this.qubit.quaternion.clone();
    const target = new Quaternion()
      .setFromAxisAngle(axis, angle)
      .multiply(initial);

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

export function BlochSphere(props: { gates: string[] }) {
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
