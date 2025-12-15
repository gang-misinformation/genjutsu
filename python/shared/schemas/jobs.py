from pydantic import BaseModel, Field
from typing import Optional
from enum import Enum

class GenerateRequest(BaseModel):
    prompt: str = Field(..., description="Text description of 3D object")
    model: str = Field(default="shap_e", description="Model to use")
    guidance_scale: float = Field(default=15.0, ge=1.0, le=30.0)
    num_inference_steps: int = Field(default=64, ge=16, le=256)

class JobStatus(str, Enum):
    PENDING = "PENDING"
    STARTED = "STARTED"
    SUCCESS = "SUCCESS"
    FAILURE = "FAILURE"
    RETRY = "RETRY"
    REVOKED = "REVOKED"

class JobResponse(BaseModel):
    job_id: str
    status: JobStatus
    message: Optional[str] = None

class GenerationResult(BaseModel):
    ply_path: str

class JobStatusResponse(BaseModel):
    job_id: str
    status: JobStatus
    progress: Optional[float] = None
    message: Optional[str] = None
    result: Optional[GenerationResult] = None
    error: Optional[str] = None