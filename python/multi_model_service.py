#!/usr/bin/env python3
"""
Multi-Model 3D Generation Service
Supports: DreamScene360, GaussianDreamerPro, TripoSR, SceneScape
"""

import argparse
import os
from flask import Flask, request, jsonify
from pathlib import Path
import torch
from datetime import datetime
import numpy as np

app = Flask(__name__)

class Model3DService:
    """Manages multiple text-to-3D models"""

    def __init__(self, output_dir="./outputs"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        self.models = {}
        self.load_models()

    def load_models(self):
        """Load available models"""
        print("Loading models...")

        # Try loading DreamScene360
        try:
            # from dreamscene360 import DreamScene360Pipeline
            # self.models['dreamscene360'] = DreamScene360Pipeline()
            print("  ⚠️  DreamScene360: Not installed (placeholder)")
            self.models['dreamscene360'] = None
        except:
            print("  ✗ DreamScene360: Not available")
            self.models['dreamscene360'] = None

        # Try loading GaussianDreamerPro
        try:
            # from gaussiandreamer import GaussianDreamerProPipeline
            # self.models['gaussiandreamerpro'] = GaussianDreamerProPipeline()
            print("  ⚠️  GaussianDreamerPro: Not installed (placeholder)")
            self.models['gaussiandreamerpro'] = None
        except:
            print("  ✗ GaussianDreamerPro: Not available")
            self.models['gaussiandreamerpro'] = None

        # Try loading TripoSR
        try:
            # from triposr import TripoSRPipeline
            # self.models['triposr'] = TripoSRPipeline()
            print("  ⚠️  TripoSR: Not installed (placeholder)")
            self.models['triposr'] = None
        except:
            print("  ✗ TripoSR: Not available")
            self.models['triposr'] = None

        # Try loading SceneScape
        try:
            # from scenescape import SceneScapePipeline
            # self.models['scenescape'] = SceneScapePipeline()
            print("  ⚠️  SceneScape: Not installed (placeholder)")
            self.models['scenescape'] = None
        except:
            print("  ✗ SceneScape: Not available")
            self.models['scenescape'] = None

        print(f"✓ Service initialized with {len([m for m in self.models.values() if m])} active models")

    def generate(self, prompt, model_name='dreamscene360', **kwargs):
        """
        Generate 3D content using specified model

        Args:
            prompt: Text description
            model_name: Which model to use
            **kwargs: Model-specific parameters

        Returns:
            path to generated .ply file
        """
        print(f"[{model_name}] Generating: '{prompt}'")

        # Create output filename
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        safe_prompt = "".join(c for c in prompt if c.isalnum() or c in (' ', '_')).rstrip()
        safe_prompt = safe_prompt[:50].replace(' ', '_')
        output_name = f"{model_name}_{safe_prompt}_{timestamp}"
        output_path = self.output_dir / f"{output_name}.ply"

        # Route to appropriate model
        if model_name == 'dreamscene360':
            return self._generate_dreamscene360(output_path, prompt, **kwargs)
        elif model_name == 'gaussiandreamerpro':
            return self._generate_gaussiandreamerpro(output_path, prompt, **kwargs)
        elif model_name == 'triposr':
            return self._generate_triposr(output_path, prompt, **kwargs)
        elif model_name == 'scenescape':
            return self._generate_scenescape(output_path, prompt, **kwargs)
        else:
            raise ValueError(f"Unknown model: {model_name}")

    def _generate_dreamscene360(self, output_path, prompt, **kwargs):
        """Generate 360° scene with DreamScene360"""
        model = self.models.get('dreamscene360')

        if model:
            # Real generation
            result = model(
                prompt=prompt,
                guidance_scale=kwargs.get('guidance_scale', 7.5),
                num_inference_steps=kwargs.get('num_inference_steps', 50),
                output_path=str(output_path)
            )
            return str(output_path)
        else:
            # Placeholder: Generate sphere scene
            return self._placeholder_scene(output_path, prompt, scene_type='sphere')

    def _generate_gaussiandreamerpro(self, output_path, prompt, **kwargs):
        """Generate high-quality object with GaussianDreamerPro"""
        model = self.models.get('gaussiandreamerpro')

        if model:
            result = model(
                prompt=prompt,
                guidance_scale=kwargs.get('guidance_scale', 7.5),
                num_iterations=kwargs.get('num_iterations', 500),
                output_path=str(output_path)
            )
            return str(output_path)
        else:
            # Placeholder: Generate object
            return self._placeholder_scene(output_path, prompt, scene_type='object')

    def _generate_triposr(self, output_path, prompt, **kwargs):
        """Generate fast preview with TripoSR"""
        model = self.models.get('triposr')

        if model:
            # TripoSR: text -> image -> 3D
            result = model(
                prompt=prompt,
                output_path=str(output_path)
            )
            return str(output_path)
        else:
            # Placeholder: Quick sphere
            return self._placeholder_scene(output_path, prompt, scene_type='preview')

    def _generate_scenescape(self, output_path, prompt, **kwargs):
        """Generate complex scene with SceneScape"""
        model = self.models.get('scenescape')

        if model:
            result = model(
                prompt=prompt,
                num_objects=kwargs.get('num_objects', 5),
                output_path=str(output_path)
            )
            return str(output_path)
        else:
            # Placeholder: Multi-object scene
            return self._placeholder_scene(output_path, prompt, scene_type='complex')

    def _placeholder_scene(self, output_path, prompt, scene_type='sphere'):
        """Generate placeholder .ply for testing"""
        print(f"  → Generating placeholder ({scene_type})")

        if scene_type == 'sphere':
            # Single sphere of points
            num_points = 10000
            phi = np.random.uniform(0, np.pi, num_points)
            theta = np.random.uniform(0, 2 * np.pi, num_points)
            r = np.random.uniform(0.8, 1.2, num_points)

        elif scene_type == 'object':
            # Denser, smaller object
            num_points = 5000
            phi = np.random.uniform(0, np.pi, num_points)
            theta = np.random.uniform(0, 2 * np.pi, num_points)
            r = np.random.uniform(0.3, 0.5, num_points)

        elif scene_type == 'preview':
            # Very simple, fast
            num_points = 1000
            phi = np.random.uniform(0, np.pi, num_points)
            theta = np.random.uniform(0, 2 * np.pi, num_points)
            r = np.ones(num_points) * 0.5

        elif scene_type == 'complex':
            # Multiple spheres at different positions
            num_spheres = 3
            points_per_sphere = 3000
            num_points = num_spheres * points_per_sphere

            positions = []
            for i in range(num_spheres):
                center = np.array([
                    (i - 1) * 0.8,  # Spread horizontally
                    np.random.uniform(-0.3, 0.3),
                    np.random.uniform(-0.3, 0.3)
                ])

                phi = np.random.uniform(0, np.pi, points_per_sphere)
                theta = np.random.uniform(0, 2 * np.pi, points_per_sphere)
                r = np.random.uniform(0.2, 0.3, points_per_sphere)

                x = r * np.sin(phi) * np.cos(theta) + center[0]
                y = r * np.sin(phi) * np.sin(theta) + center[1]
                z = r * np.cos(phi) + center[2]

                positions.append(np.stack([x, y, z], axis=1))

            positions = np.vstack(positions)
            x, y, z = positions[:, 0], positions[:, 1], positions[:, 2]

            # Write and return early
            return self._write_ply(output_path, x, y, z, num_points, prompt)

        else:
            raise ValueError(f"Unknown scene type: {scene_type}")

        # Convert spherical to cartesian (for non-complex scenes)
        x = r * np.sin(phi) * np.cos(theta)
        y = r * np.sin(phi) * np.sin(theta)
        z = r * np.cos(phi)

        return self._write_ply(output_path, x, y, z, num_points, prompt)

    def _write_ply(self, output_path, x, y, z, num_points, prompt):
        """Write PLY file with Gaussian splat format"""
        # Color based on prompt keywords
        if "red" in prompt.lower():
            base_color = np.array([255, 100, 100])
        elif "blue" in prompt.lower():
            base_color = np.array([100, 150, 255])
        elif "green" in prompt.lower():
            base_color = np.array([100, 255, 150])
        elif "yellow" in prompt.lower():
            base_color = np.array([255, 255, 100])
        else:
            base_color = np.array([180, 180, 200])

        color_array = np.tile(base_color, (num_points, 1))
        color_array += np.random.randint(-30, 30, (num_points, 3))
        color_array = np.clip(color_array, 0, 255).astype(np.uint8)

        scale = np.ones((num_points, 3)) * 0.03
        rotation = np.tile([1, 0, 0, 0], (num_points, 1))
        opacity = np.ones(num_points) * 0.7

        with open(output_path, 'wb') as f:
            header = f"""ply
format binary_little_endian 1.0
element vertex {num_points}
property float x
property float y
property float z
property float nx
property float ny
property float nz
property uchar red
property uchar green
property uchar blue
property float opacity
property float scale_0
property float scale_1
property float scale_2
property float rot_0
property float rot_1
property float rot_2
property float rot_3
end_header
"""
            f.write(header.encode('ascii'))

            for i in range(num_points):
                f.write(x[i].astype(np.float32).tobytes())
                f.write(y[i].astype(np.float32).tobytes())
                f.write(z[i].astype(np.float32).tobytes())
                f.write(np.float32(0).tobytes() * 3)  # normals
                f.write(bytes(color_array[i]))
                f.write(opacity[i].astype(np.float32).tobytes())
                f.write(scale[i].astype(np.float32).tobytes())
                f.write(rotation[i].astype(np.float32).tobytes())

        print(f"  ✓ Generated: {output_path}")
        return str(output_path)

# Global service
service = None

@app.route('/health', methods=['GET'])
def health():
    """Health check"""
    available_models = [name for name, model in service.models.items() if model is not None]

    return jsonify({
        "status": "healthy",
        "available_models": available_models,
        "placeholder_models": [name for name in service.models.keys() if service.models[name] is None],
        "device": "cuda" if torch.cuda.is_available() else "cpu"
    })

@app.route('/models', methods=['GET'])
def list_models():
    """List available models and their info"""
    models_info = {
        'dreamscene360': {
            'name': 'DreamScene360',
            'type': 'scene',
            'speed': '2-5 min',
            'quality': 'high',
            'available': service.models['dreamscene360'] is not None
        },
        'gaussiandreamerpro': {
            'name': 'GaussianDreamerPro',
            'type': 'object',
            'speed': '10-15 min',
            'quality': 'very_high',
            'available': service.models['gaussiandreamerpro'] is not None
        },
        'triposr': {
            'name': 'TripoSR',
            'type': 'object',
            'speed': '<1 sec',
            'quality': 'medium',
            'available': service.models['triposr'] is not None
        },
        'scenescape': {
            'name': 'SceneScape',
            'type': 'scene',
            'speed': '3-7 min',
            'quality': 'high',
            'available': service.models['scenescape'] is not None
        }
    }

    return jsonify(models_info)

@app.route('/generate', methods=['POST'])
def generate():
    """
    Generate 3D content

    POST body:
    {
        "prompt": "a medieval castle",
        "model": "dreamscene360",
        "guidance_scale": 7.5,
        "num_inference_steps": 50
    }
    """
    data = request.json
    prompt = data.get('prompt')
    model_name = data.get('model', 'dreamscene360')

    if not prompt:
        return jsonify({"error": "prompt is required"}), 400

    if model_name not in service.models:
        return jsonify({
            "error": f"Unknown model: {model_name}",
            "available_models": list(service.models.keys())
        }), 400

    try:
        # Extract model-specific parameters
        kwargs = {
            'guidance_scale': data.get('guidance_scale', 7.5),
            'num_inference_steps': data.get('num_inference_steps', 50),
            'num_iterations': data.get('num_iterations', 500),
            'num_objects': data.get('num_objects', 5)
        }

        output_path = service.generate(prompt, model_name, **kwargs)

        return jsonify({
            "status": "success",
            "output_path": output_path,
            "model": model_name,
            "prompt": prompt,
            "placeholder": service.models[model_name] is None
        })

    except Exception as e:
        return jsonify({
            "status": "error",
            "error": str(e)
        }), 500

def main():
    parser = argparse.ArgumentParser(description="Multi-Model 3D Generation Service")
    parser.add_argument('--host', default='127.0.0.1')
    parser.add_argument('--port', type=int, default=5000)
    parser.add_argument('--output-dir', default='../outputs')

    args = parser.parse_args()

    global service
    service = Model3DService(output_dir=args.output_dir)

    print(f"""
╔════════════════════════════════════════════════════════════╗
║        Multi-Model 3D Generation Service                   ║
╚════════════════════════════════════════════════════════════╝

Listening: http://{args.host}:{args.port}
Output: {args.output_dir}

Endpoints:
  GET  /health     - Service health + available models
  GET  /models     - Model information
  POST /generate   - Generate 3D from text

Models:
  • dreamscene360      - 360° scenes (PRIMARY)
  • gaussiandreamerpro - High-quality objects
  • triposr           - Fast previews
  • scenescape        - Complex multi-object scenes
""")

    app.run(host=args.host, port=args.port, debug=False)

if __name__ == '__main__':
    main()