#!/usr/bin/env python3
"""
Multi-Model 3D Generation Service
Routes requests to appropriate model implementations
"""

import argparse
from pathlib import Path
from datetime import datetime
from flask import Flask, request, jsonify
import torch

from models.shap_e import ShapEModel


app = Flask(__name__)


class MultiModelService:
    """Service that manages multiple 3D generation models"""

    def __init__(self, output_dir="./outputs"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

        self.device = "cuda" if torch.cuda.is_available() else "cpu"
        print(f"Using device: {self.device}")

        # Initialize models
        self.models = {}
        self.load_all_models()

    def load_all_models(self):
        """Try to load all available models"""
        print("\nLoading models...")

        # Try Shap-E
        print("1. Shap-E:")
        shap_e = ShapEModel(self.device)
        if shap_e.load():
            self.models['shap_e'] = shap_e

        # Summary
        print(f"\n✓ Loaded {len(self.models)} model(s): {list(self.models.keys())}")

        if not self.models:
            print("⚠️  No models loaded! Service will return errors.")

    def generate(self, prompt, model_name, **kwargs):
        """
        Generate 3D content with specified model

        Args:
            prompt: Text description
            model_name: Which model to use (shap_e)
            **kwargs: Model-specific parameters
        """
        # Check model exists
        if model_name not in self.models:
            available = list(self.models.keys())
            raise ValueError(
                f"Model '{model_name}' not available. "
                f"Available models: {available}"
            )

        # Create output path
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        safe_prompt = "".join(c for c in prompt if c.isalnum() or c in (' ', '_')).strip()
        safe_prompt = safe_prompt[:50].replace(' ', '_')

        output_name = f"{model_name}_{safe_prompt}_{timestamp}.ply"
        output_path = self.output_dir / output_name

        # Generate
        model = self.models[model_name]
        print(f"\n{'='*60}")
        print(f"Model: {model.get_name()}")
        print(f"Prompt: {prompt}")
        print(f"Output: {output_path}")
        print(f"{'='*60}\n")

        result_path = model.generate(prompt, output_path, **kwargs)

        return str(result_path)


# Global service instance
service = None


@app.route('/health', methods=['GET'])
def health():
    """Health check"""
    return jsonify({
        "status": "healthy",
        "device": service.device,
        "models": {
            name: {
                "name": model.get_name(),
                "loaded": model.is_loaded
            }
            for name, model in service.models.items()
        }
    })


@app.route('/models', methods=['GET'])
def list_models():
    """List available models"""
    models_info = {}

    for name, model in service.models.items():
        models_info[name] = {
            "name": model.get_name(),
            "loaded": model.is_loaded,
            "estimated_time_sec": model.get_estimated_time()
        }

    return jsonify(models_info)


@app.route('/generate', methods=['POST'])
def generate():
    """
    Generate 3D content

    POST body:
    {
        "prompt": "a red sports car",
        "model": "shap_e",
        "guidance_scale": 15.0,
        "num_inference_steps": 64
    }
    """
    data = request.json

    prompt = data.get('prompt')
    model_name = data.get('model', 'shap_e')

    if not prompt:
        return jsonify({"error": "prompt is required"}), 400

    try:
        # Extract model-specific params
        kwargs = {
            'guidance_scale': data.get('guidance_scale', 15.0),
            'num_inference_steps': data.get('num_inference_steps', 64),
        }

        # Generate
        output_path = service.generate(prompt, model_name, **kwargs)

        return jsonify({
            "status": "success",
            "output_path": output_path,
            "model": model_name,
            "prompt": prompt
        })

    except ValueError as e:
        return jsonify({"status": "error", "error": str(e)}), 400
    except Exception as e:
        import traceback
        traceback.print_exc()
        return jsonify({"status": "error", "error": str(e)}), 500


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--host', default='127.0.0.1')
    parser.add_argument('--port', type=int, default=5000)
    parser.add_argument('--output-dir', default='../outputs')

    args = parser.parse_args()

    global service
    service = MultiModelService(output_dir=args.output_dir)

    print(f"""
╔══════════════════════════════════════════════════════════╗
║          Multi-Model 3D Generation Service               ║
╚══════════════════════════════════════════════════════════╝

Listening: http://{args.host}:{args.port}
Output: {args.output_dir}

Endpoints:
  GET  /health    - Service health + loaded models
  GET  /models    - List available models
  POST /generate  - Generate 3D from text

Loaded models: {list(service.models.keys())}
""")

    app.run(host=args.host, port=args.port, debug=False, threaded=True)


if __name__ == '__main__':
    main()