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

def update_job_status(job_id: str, metadata: dict, outputs: dict = None):
    """Notify Rust app of job status via HTTP callback

    The Rust backend expects:
    - id: job_id
    - data: JobMetadata (status, progress, message, etc.)
    - outputs: Optional JobOutputs (ply_path)
    """
    try:
        # Build payload matching Rust JobStatusResponse schema
        payload = {
            "id": job_id,
            "data": metadata,
            "outputs": outputs
        }

        print(f"Sending callback to {RUST_CALLBACK_URL}/job/{job_id}/progress")
        print(f"Payload: {payload}")

        response = requests.post(
            f"{RUST_CALLBACK_URL}/job/{job_id}/progress",
            json=payload,
            timeout=5
        )

        if response.status_code != 200:
            print(f"Warning: Callback failed with status {response.status_code}: {response.text}")

    except requests.exceptions.RequestException as e:
        print(f"Warning: Failed to notify Rust app: {e}")
        # Don't fail the job if callback fails


@celery_app.task(name='worker.generate_3d', bind=True)
def generate_3d(self, prompt: str, model_name: str, guidance_scale: float, num_inference_steps: int):
    """
    Generate 3D model from text prompt

    Args:
        self: Task instance (for progress updates)
        prompt: Text description
        model_name: Model to use
        guidance_scale: Guidance scale parameter
        num_inference_steps: Number of diffusion steps

    Returns:
        dict with output_path and metadata
    """
    job_id = self.request.id

    try:
        # Check model exists
        if model_name not in MODELS:
            error_msg = f"Model '{model_name}' not available"
            # Send failure status
            update_job_status(
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

        # Initial status update - GENERATING
        update_job_status(
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

        # Progress callback
        def progress_callback(progress: float, message: str):
            update_job_status(
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
            # Generation failed - return helpful error
            error_msg = str(e)
            if "flat" in error_msg.lower() or "degenerate" in error_msg.lower():
                error_msg = (
                    f"Generated mesh is flat/invalid. Try:\n"
                    f"  • More specific prompt (add details about shape/structure)\n"
                    f"  • Higher guidance scale (try 20-25)\n"
                    f"  • Different prompt entirely"
                )

            update_job_status(
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
        # The path should be relative to project root
        relative_path = str(result_path.relative_to(OUTPUT_DIR.parent))

        print(f"Relative path for Rust: {relative_path}")

        # Success - notify with result
        update_job_status(
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

        update_job_status(
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