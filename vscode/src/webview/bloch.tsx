// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* TODO:

- Move to the npm package as a control
- Put data on the screen. See https://threejs.org/manual/#en/debugging-javascript
- Add the trailing dots with a slider for history (fade out speed)
- Add a few canonical gates to apply

*/

import { useEffect, useRef } from "preact/hooks";

import {
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
  Scene,
  SphereGeometry,
  WebGLRenderer,
  WireframeGeometry,
} from "three";

import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";

export function BlochSphere() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => renderScene(), []);

  function renderScene() {
    const canvas = canvasRef.current;
    if (!canvas) return;

    // const backgroundColor = window
    //   .getComputedStyle(canvas)
    //   .getPropertyValue("--main-background");

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

    // In WebGL, Z is towards the camera (looking at -Z), Y is up, X is right
    camera.position.z = 27;

    const light = new DirectionalLight(0xffffff, 0.5);
    light.position.set(-1, 2, 4);
    scene.add(light);

    const controls = new OrbitControls(camera, renderer.domElement);
    // Add the main sphere
    const sphereGeometry = new SphereGeometry(5, 32, 16);
    const material = new MeshLambertMaterial({
      emissive: 0x404080,
      emissiveIntensity: 3,
      transparent: true,
      opacity: 0.5,
    });
    const sphere = new Mesh(sphereGeometry, material);
    scene.add(sphere);

    // Draw the wires on it
    const sphereWireGeometry = new SphereGeometry(5, 16, 16);
    const wireframe = new WireframeGeometry(sphereWireGeometry);
    const sphereLines = new LineSegments(wireframe);
    const materialProps = sphereLines.material as MeshBasicMaterialParameters;
    materialProps.depthTest = false;
    materialProps.opacity = 0.1;
    materialProps.transparent = true;
    scene.add(sphereLines);

    // Add the axes
    const axisMaterial = new MeshBasicMaterial({ color: 0xe0d0c0 });
    const zAxis = new CylinderGeometry(0.075, 0.075, 12, 32, 8);
    const zAxisMesh = new Mesh(zAxis, axisMaterial);
    scene.add(zAxisMesh);

    const zPointer = new ConeGeometry(0.2, 0.8, 16);
    const zPointerMesh = new Mesh(zPointer, axisMaterial);
    zPointerMesh.position.set(0, 6, 0);
    scene.add(zPointerMesh);

    // TODO: Duplicate the above for the X and Y axes
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

    // Add the points from |0> to |1>
    const curve = new EllipseCurve(0, 0, 5.05, 5.05, 0, Math.PI);
    const points = curve.getPoints(32);
    const pointMaterial = new MeshBasicMaterial({ color: 0x80ffd0 });

    const pointsGroup = new Group();
    for (const point of points) {
      const pointGeometry = new SphereGeometry(0.1, 16, 16);
      const pointMesh = new Mesh(pointGeometry, pointMaterial);
      pointMesh.position.set(0, point.x, point.y);
      pointsGroup.add(pointMesh);
    }
    pointsGroup.rotateY(0.5);
    // pointsGroup.rotateZ(Math.PI / 2);
    // pointsGroup.rotateX(0.5);
    scene.add(pointsGroup);

    function animate() {
      controls.update();
      renderer.render(scene, camera);
      // requestAnimationFrame(animate);
    }

    // See https://threejs.org/manual/#en/rendering-on-demand
    controls.addEventListener("change", () => requestAnimationFrame(animate));

    requestAnimationFrame(animate);
  }

  return <canvas ref={canvasRef} width="600" height="600"></canvas>;
}
