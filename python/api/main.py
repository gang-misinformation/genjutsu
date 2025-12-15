"""
FastAPI service for job submission and status
"""
from fastapi import FastAPI, HTTPException
import sys
from pathlib import Path

# Add parent directory to path for shared module
sys.path.insert(0, str(Path(__file__).parent))

from shared.celery_app import celery_app
from shared.config import OUTPUT_DIR
from shared.schemas.jobs import *

app = FastAPI(title="Genjutsu 3D Generation API")

@app.get("/")
async def root():
    return {
        "service": "Genjutsu 3D Generation API",
        "version": "2.0",
        "docs": "/docs"
    }


@app.get("/health")
async def health():
    """Health check endpoint"""
    # Check Redis connection
    try:
        celery_app.broker_connection().ensure_connection(max_retries=3)
        redis_status = "connected"
    except Exception as e:
        redis_status = f"disconnected: {str(e)}"

    # Check worker availability
    stats = celery_app.control.inspect().stats()
    active_workers = len(stats) if stats else 0

    return {
        "status": "healthy" if redis_status == "connected" else "degraded",
        "redis": redis_status,
        "workers": active_workers,
        "output_dir": str(OUTPUT_DIR)
    }


@app.get("/workers")
async def list_workers():
    """List active Celery workers"""
    stats = celery_app.control.inspect().stats()
    active = celery_app.control.inspect().active()

    return {
        "workers": stats or {},
        "active_tasks": active or {}
    }


@app.post("/generate", response_model=JobResponse)
async def generate(request: GenerateRequest):
    """
    Submit a 3D generation job
    
    Returns job_id for tracking progress
    """
    from worker.tasks import generate_3d
    try:
        # Submit task to Celery
        task = generate_3d.delay(
            request.prompt,
            request.model,
            request.guidance_scale,
            request.num_inference_steps
        )

        return JobResponse(
            job_id=task.id,
            status=JobStatus.STARTED,
            message=f"Job submitted for model '{request.model}'"
        )

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/status/{job_id}", response_model=JobStatusResponse)
async def get_status(job_id: str):
    """
    Get job status and result
    """
    try:
        result = celery_app.AsyncResult(job_id)

        response = JobStatusResponse(
            job_id=job_id,
            status=result.state
        )

        if result.state == JobStatus.PENDING.value:
            response.message = "Job is queued"

        elif result.state == JobStatus.STARTED.value:
            # Get progress if available
            if result.info and isinstance(result.info, dict):
                response.progress = result.info.get('progress', 0.0)
                response.message = result.info.get('message', 'Processing...')
            else:
                response.message = "Job started"

        elif result.state ==  JobStatus.SUCCESS.value:
            response.message = "Generation complete"
            response.progress = 1.0
            response.result = result.result

        elif result.state == JobStatus.FAILURE.value:
            response.message = "Job failed"
            response.error = str(result.info)

        elif result.state == JobStatus.RETRY.value:
            response.message = "Job is being retried"

        return response

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.delete("/cancel/{job_id}")
async def cancel_job(job_id: str):
    """Cancel a running job"""
    try:
        celery_app.control.revoke(job_id, terminate=True)
        return {"job_id": job_id, "status": JobStatus.REVOKED.value}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.get("/queue")
async def queue_info():
    """Get queue statistics"""
    inspect = celery_app.control.inspect()

    return {
        "active": inspect.active() or {},
        "scheduled": inspect.scheduled() or {},
        "reserved": inspect.reserved() or {}
    }


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=5000)