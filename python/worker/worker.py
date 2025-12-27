"""
Celery worker for 3D generation tasks
"""
import sys
from pathlib import Path
from datetime import datetime
import os

import requests

# Add parent directory to path for shared module
sys.path.insert(0, str(Path(__file__).parent.parent))

from shared.celery_app import celery_app
from shared.config import OUTPUT_DIR, DEVICE
from shared.schemas.jobs import JobStatus
from models.shap_e import ShapEModel

# Load models on worker startup
print("=" * 60)
print("Initializing Genjutsu Worker")
print("=" * 60)
print(f"Device: {DEVICE}")
print(f"Output: {OUTPUT_DIR}")
print()

# Initialize models
MODELS = {}

print("Loading Shap-E...")
shap_e = ShapEModel(DEVICE)
if shap_e.load():
    MODELS['shap_e'] = shap_e
    print("✓ Shap-E ready")
else:
    print("✗ Shap-E failed to load")

print()
print(f"Loaded {len(MODELS)} model(s)")
print("=" * 60)
print()

# Rust backend callback URL (using host.docker.internal from docker-compose)
RUST_CALLBACK_URL = os.getenv('RUST_CALLBACK_URL', 'http://host.docker.internal:3000')


def notify_rust_only(job_id: str, metadata: dict, outputs: dict = None):
    """Send status update to Rust app for UI display (no DB write)

    This just updates the UI state, not the persistent database.
    """
    try:
        payload = {
            "id": job_id,
            "data": metadata,
            "outputs": outputs
        }

        response = requests.post(
            f"{RUST_CALLBACK_URL}/job/{job_id}/progress",
            json=payload,
            timeout=5
        )

        if response.status_code != 200:
            print(f"Warning: Callback failed with status {response.status_code}")

    except requests.exceptions.RequestException as e:
        print(f"Warning: Failed to notify Rust app: {e}")


@celery_app.task(name='worker.generate_3d', bind=True)
def generate_3d(self, prompt: str, model_name: str, guidance_scale: float, num_inference_steps: int):
    """
    Generate 3D model from text prompt

    Database updates only happen at:
    - Job start (GENERATING)
    - Job completion (COMPLETE with outputs)
    - Job failure (FAILED with error)

    Progress updates go directly to Rust app for UI only.
    """
    job_id = self.request.id

    try:
        # Check model exists
        if model_name not in MODELS:
            error_msg = f"Model '{model_name}' not available"
            # This updates DB via Rust
            notify_rust_only(
                job_id,
                metadata={
                    "status": JobStatus.FAILED.value,
                    "progress": 0.0,
                    "message": None,
                    "error": error_msg,
                    "created_at": datetime.utcnow().isoformat() + 'Z',
                    "updated_at": datetime.utcnow().isoformat() + 'Z',
                    "completed_at": datetime.utcnow().isoformat() + 'Z'
                }
            )
            raise ValueError(error_msg)

        model = MODELS[model_name]

        # Create output path
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        safe_prompt = "".join(c for c in prompt if c.isalnum() or c in (' ', '_')).strip()
        safe_prompt = safe_prompt[:50].replace(' ', '_')

        output_name = f"{model_name}_{safe_prompt}_{timestamp}.ply"
        output_path = OUTPUT_DIR / output_name

        print(f"\n{'='*60}")
        print(f"Job ID: {job_id}")
        print(f"Model: {model.get_name()}")
        print(f"Prompt: {prompt}")
        print(f"Output: {output_path}")
        print(f"Guidance: {guidance_scale}")
        print(f"Steps: {num_inference_steps}")
        print(f"{'='*60}\n")

        # DB UPDATE #1: Job started
        notify_rust_only(
            job_id,
            metadata={
                "status": JobStatus.GENERATING.value,
                "progress": 0.0,
                "message": "Initializing generation...",
                "error": None,
                "created_at": datetime.utcnow().isoformat() + 'Z',
                "updated_at": datetime.utcnow().isoformat() + 'Z',
                "completed_at": None
            }
        )

        # Progress callback - ONLY updates Rust UI, NOT database
        def progress_callback(progress: float, message: str):
            notify_rust_only(
                job_id,
                metadata={
                    "status": JobStatus.GENERATING.value,
                    "progress": progress,
                    "message": message,
                    "error": None,
                    "created_at": datetime.utcnow().isoformat() + 'Z',
                    "updated_at": datetime.utcnow().isoformat() + 'Z',
                    "completed_at": None
                }
            )
            print(f"[{progress*100:.0f}%] {message}")

        progress_callback(0.1, 'Starting generation...')

        # Generate
        try:
            result_path = model.generate(
                prompt,
                output_path,
                guidance_scale=guidance_scale,
                num_inference_steps=num_inference_steps
            )

            # Verify file exists
            if not result_path.exists():
                raise FileNotFoundError(f"Output file not created: {result_path}")

            print(f"\n✓ Generation complete: {result_path}")

        except ValueError as e:
            # Generation failed
            error_msg = str(e)
            if "flat" in error_msg.lower() or "degenerate" in error_msg.lower():
                error_msg = (
                    f"Generated mesh is flat/invalid. Try:\n"
                    f"  • More specific prompt (add details about shape/structure)\n"
                    f"  • Higher guidance scale (try 20-25)\n"
                    f"  • Different prompt entirely"
                )

            # DB UPDATE #2: Job failed
            notify_rust_only(
                job_id,
                metadata={
                    "status": JobStatus.FAILED.value,
                    "progress": 0.0,
                    "message": None,
                    "error": error_msg,
                    "created_at": datetime.utcnow().isoformat() + 'Z',
                    "updated_at": datetime.utcnow().isoformat() + 'Z',
                    "completed_at": datetime.utcnow().isoformat() + 'Z'
                }
            )
            raise ValueError(error_msg)

        # Get relative path for Rust app
        relative_path = str(result_path.relative_to(OUTPUT_DIR.parent))

        print(f"Relative path for Rust: {relative_path}")

        # DB UPDATE #3: Job completed successfully
        notify_rust_only(
            job_id,
            metadata={
                "status": JobStatus.COMPLETE.value,
                "progress": 1.0,
                "message": "Generation complete!",
                "error": None,
                "created_at": datetime.utcnow().isoformat() + 'Z',
                "updated_at": datetime.utcnow().isoformat() + 'Z',
                "completed_at": datetime.utcnow().isoformat() + 'Z'
            },
            outputs={
                "ply_path": relative_path
            }
        )

        return {
            "ply_path": relative_path
        }

    except Exception as e:
        error_msg = str(e)
        print(f"\n✗ Job failed: {error_msg}\n")

        # DB UPDATE #4: Unexpected failure
        notify_rust_only(
            job_id,
            metadata={
                "status": JobStatus.FAILED.value,
                "progress": 0.0,
                "message": None,
                "error": error_msg,
                "created_at": datetime.utcnow().isoformat() + 'Z',
                "updated_at": datetime.utcnow().isoformat() + 'Z',
                "completed_at": datetime.utcnow().isoformat() + 'Z'
            }
        )
        raise


if __name__ == '__main__':
    # Start worker
    celery_app.worker_main([
        'worker',
        '--loglevel=info',
        '--concurrency=1',  # Single worker (GPU)
        '--pool=solo'  # Use solo pool for GPU work
    ])