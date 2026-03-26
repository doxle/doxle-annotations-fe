// Doxle 3D Viewer — Three.js rendering layer
// Called from Rust/WASM via window.doxleViewer.*

(function () {
  let scene, camera, renderer, controls;
  let wallGroup = null;
  let animationId = null;

  function animate() {
    animationId = requestAnimationFrame(animate);
    if (controls) controls.update();
    if (renderer && scene && camera) {
      renderer.render(scene, camera);
    }
  }

  function onResize() {
    const container = renderer?.domElement?.parentElement;
    if (!container || !camera || !renderer) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    renderer.setSize(w, h);
  }

  window.doxleViewer = {
    initViewer: function (containerId) {
      const THREE = window.THREE;
      if (!THREE) {
        console.error('[doxleViewer] THREE not loaded');
        return;
      }

      const container = document.getElementById(containerId);
      if (!container) {
        console.error('[doxleViewer] container not found:', containerId);
        return;
      }

      // Scene
      scene = new THREE.Scene();
      scene.background = new THREE.Color(0x000000);

      // Camera — position to see a ~3.6m wall nicely
      const w = container.clientWidth;
      const h = container.clientHeight;
      camera = new THREE.PerspectiveCamera(50, w / h, 0.01, 100);
      camera.position.set(6, 4, 6);
      camera.lookAt(0, 1.2, 0);

      // Renderer
      renderer = new THREE.WebGLRenderer({ antialias: true });
      renderer.setSize(w, h);
      renderer.setPixelRatio(window.devicePixelRatio);
      container.appendChild(renderer.domElement);

      // OrbitControls
      const OrbitControls = window.DoxleOrbitControls;
      controls = new OrbitControls(camera, renderer.domElement);
      controls.target.set(0, 1.2, 0);
      controls.enableDamping = true;
      controls.dampingFactor = 0.1;
      controls.update();

      // Lights
      const ambient = new THREE.AmbientLight(0xffffff, 0.6);
      scene.add(ambient);
      const dir = new THREE.DirectionalLight(0xffffff, 0.8);
      dir.position.set(5, 8, 5);
      scene.add(dir);

      // Wall group
      wallGroup = new THREE.Group();
      scene.add(wallGroup);

      window.addEventListener('resize', onResize);

      // Measurement tool
      var measureState = { points: [], markers: [], line: null, label: null };
      var raycaster = new THREE.Raycaster();
      var mouse = new THREE.Vector2();

      function clearMeasure() {
        measureState.markers.forEach(function (mk) { scene.remove(mk); mk.geometry.dispose(); mk.material.dispose(); });
        measureState.markers = [];
        measureState.points = [];
        if (measureState.line) { scene.remove(measureState.line); measureState.line.geometry.dispose(); measureState.line.material.dispose(); measureState.line = null; }
        var el = document.getElementById('doxle-measure-label');
        if (el) el.textContent = '';
      }

      function addMarker(pt) {
        var g = new THREE.SphereGeometry(0.02, 12, 12);
        var mt = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
        var sp = new THREE.Mesh(g, mt);
        sp.position.copy(pt);
        scene.add(sp);
        measureState.markers.push(sp);
      }

      function handleMeasureHit(clientX, clientY) {
        var rect = renderer.domElement.getBoundingClientRect();
        mouse.x = ((clientX - rect.left) / rect.width) * 2 - 1;
        mouse.y = -((clientY - rect.top) / rect.height) * 2 + 1;
        raycaster.setFromCamera(mouse, camera);
        var hits = raycaster.intersectObjects(wallGroup.children, true);
        if (hits.length === 0) return;
        var pt = hits[0].point.clone();

        if (measureState.points.length >= 2) clearMeasure();

        measureState.points.push(pt);
        addMarker(pt);

        if (measureState.points.length === 2) {
          var a = measureState.points[0], b = measureState.points[1];
          var lineGeo = new THREE.BufferGeometry().setFromPoints([a, b]);
          var lineMat = new THREE.LineBasicMaterial({ color: 0x00ff00 });
          measureState.line = new THREE.Line(lineGeo, lineMat);
          scene.add(measureState.line);
          var dist = a.distanceTo(b) * 1000; // metres to mm
          var el = document.getElementById('doxle-measure-label');
          if (el) el.textContent = Math.round(dist) + ' mm';
        }
      }

      // Track recent touch to ignore synthetic mouse events on mobile
      var lastTouchTime = 0;

      // Desktop: single click to place measure points
      var clickStart = { x: 0, y: 0 };
      renderer.domElement.addEventListener('mousedown', function (e) {
        clickStart.x = e.clientX;
        clickStart.y = e.clientY;
      });
      renderer.domElement.addEventListener('mouseup', function (e) {
        if (Date.now() - lastTouchTime < 500) return;
        var dx = e.clientX - clickStart.x;
        var dy = e.clientY - clickStart.y;
        if (dx * dx + dy * dy < 25) handleMeasureHit(e.clientX, e.clientY);
      });

      // Mobile: single tap to place measure points (ignore drag/pinch)
      var touchStart = { x: 0, y: 0, time: 0 };
      renderer.domElement.addEventListener('touchstart', function (e) {
        if (e.touches.length !== 1) return;
        var t = e.touches[0];
        touchStart.x = t.clientX;
        touchStart.y = t.clientY;
        touchStart.time = Date.now();
      }, { passive: true });
      renderer.domElement.addEventListener('touchend', function (e) {
        lastTouchTime = Date.now();
        if (e.changedTouches.length !== 1) return;
        var t = e.changedTouches[0];
        var dx = t.clientX - touchStart.x;
        var dy = t.clientY - touchStart.y;
        var elapsed = Date.now() - touchStart.time;
        // Only trigger if it was a quick tap with minimal movement
        if (dx * dx + dy * dy < 100 && elapsed < 300) {
          handleMeasureHit(t.clientX, t.clientY);
        }
      });

      animate();
    },

    addWall: function (jsonStr) {
      const THREE = window.THREE;
      if (!THREE || !scene || !wallGroup) return;

      const data = JSON.parse(jsonStr);

      data.members.forEach(function (m) {
        var geo, mat, mesh;

        if (m.member_type === 'glass') {
          geo = new THREE.BoxGeometry(m.width, m.height, m.depth);
          mat = new THREE.MeshPhysicalMaterial({
            color: m.color || 0x88ccee,
            transparent: true,
            opacity: 0.3,
            roughness: 0.05,
            metalness: 0.0,
            transmission: 0.6,
          });
          mesh = new THREE.Mesh(geo, mat);
          mesh.position.set(m.x, m.y, m.z);
        } else if (m.member_type === 'pipe') {
          // Pipe: cylinder along its length axis
          var radius = m.height / 2;
          var pipeLen = m.width;
          geo = new THREE.CylinderGeometry(radius, radius, pipeLen, 16);
          mat = new THREE.MeshStandardMaterial({
            color: m.color || 0x4488cc,
            roughness: 0.3,
            metalness: 0.6,
          });
          mesh = new THREE.Mesh(geo, mat);
          mesh.position.set(m.x, m.y, m.z);
          // Rotate cylinder to lie along the correct axis
          // rotation_axis: 0=along X, 1=along Z (from Rust)
          if (m.rotation_axis === 1) {
            mesh.rotation.x = Math.PI / 2;
          } else {
            mesh.rotation.z = Math.PI / 2;
          }
        } else {
          geo = new THREE.BoxGeometry(m.width, m.height, m.depth);
          mat = new THREE.MeshStandardMaterial({
            color: m.color || 0xccaa55,
            roughness: 0.7,
            metalness: 0.1,
          });
          mesh = new THREE.Mesh(geo, mat);
          mesh.position.set(m.x, m.y, m.z);
        }

        mesh.userData = { name: m.name, type: m.member_type };
        wallGroup.add(mesh);
      });
    },

    clearScene: function () {
      if (!wallGroup) return;
      while (wallGroup.children.length > 0) {
        const child = wallGroup.children[0];
        if (child.geometry) child.geometry.dispose();
        if (child.material) child.material.dispose();
        wallGroup.remove(child);
      }
    },

    disposeViewer: function () {
      if (animationId) cancelAnimationFrame(animationId);
      window.removeEventListener('resize', onResize);
      window.doxleViewer.clearScene();
      if (renderer) {
        renderer.dispose();
        if (renderer.domElement && renderer.domElement.parentElement) {
          renderer.domElement.parentElement.removeChild(renderer.domElement);
        }
      }
      scene = null;
      camera = null;
      renderer = null;
      controls = null;
      wallGroup = null;
      animationId = null;
    },
  };
})();
