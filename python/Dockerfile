FROM nvidia/cuda:12.4.0-cudnn-devel-ubuntu22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PYTHONUNBUFFERED=1

# Install system dependencies
RUN apt-get update && apt-get install -y \
    python3.12 python3-pip git wget curl \
    && rm -rf /var/lib/apt/lists/*

# Install PyTorch with CUDA 12.4
RUN pip install torch==2.5.1 torchvision==0.20.1 \
    --index-url https://download.pytorch.org/whl/cu124

WORKDIR /app

# Copy requirements and install dependencies
COPY requirements.txt .
RUN pip install -r requirements.txt

# Copy service files
COPY models/ models/
COPY multi_model_service.py .

# Create output directory
RUN mkdir -p outputs

EXPOSE 5000

CMD ["python3", "multi_model_service.py", "--host", "0.0.0.0", "--port", "5000"]