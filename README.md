# genjutsu

A desktop app for generating interactive 3D scenes from text prompts using Gaussian Splatting rendering and OpenAI's Shap-E model.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Python](https://img.shields.io/badge/python-3670A0?style=for-the-badge&logo=python&logoColor=ffdd54)
![WebGPU](https://img.shields.io/badge/WebGPU-%23000000.svg?style=for-the-badge&logo=webgpu&logoColor=white)

![demo.png](screenshots/demo.png)

## 🎯 Features

- **✨ Text-to-3D Generation**: Create 3D models from text descriptions using Shap-E (~30-60 seconds)
- **🎨 Real-time Gaussian Splatting**: High-performance 3D rendering using WebGPU
- **🎮 Interactive Camera Controls**: Smooth rotation, zoom, and pan
- **⚡ Asynchronous Processing**: Non-blocking generation with live progress updates
- **🖥️ Cross-platform**: Works on Windows, macOS, and Linux
- **🐳 Docker Support**: Easy deployment with Docker Compose
- **📊 Job Queue System**: Redis + Celery for robust task management

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Rust Frontend (egui + winit)                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  UI (egui)   │  │ Event System │  │ Camera Ctrl  │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│              Rendering (WebGPU + wgpu)                   │
│  ┌──────────────────────────────────────────────────┐   │
│  │  Gaussian Splatting Renderer                     │   │
│  │  - Instanced quad rendering                      │   │
│  │  - Alpha blending                                │   │
│  │  - Depth sorting                                 │   │
│  └──────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────┐
│         3D Generation Pipeline (Python + Docker)         │
│                                                          │
│  Text Prompt ──────┐                                    │
│                    ▼                                     │
│         ┌──────────────────────┐                        │
│         │  FastAPI REST API    │                        │
│         │  Job Submission      │                        │
│         └──────────────────────┘                        │
│                    │                                     │
│                    ▼                                     │
│         ┌──────────────────────┐                        │
│         │  Redis Message Queue │                        │
│         └──────────────────────┘                        │
│                    │                                     │
│                    ▼                                     │
│         ┌──────────────────────┐                        │
│         │  Celery Worker       │                        │
│         │  - Shap-E Model      │                        │
│         │  - GPU Accelerated   │                        │
│         └──────────────────────┘                        │
│                    │                                     │
│                    ▼                                     │
│         ┌──────────────────────┐                        │
│         │  .ply Gaussian Cloud │                        │
│         └──────────────────────┘                        │
└─────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.75+ ([Install](https://rustup.rs/))
- **Docker & Docker Compose**: For Python service ([Install](https://docs.docker.com/get-docker/))
- **GPU**: NVIDIA GPU recommended (CPU fallback available)

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/gang-misinformation/genjutsu.git
cd genjutsu
```

2. **Create outputs directory**
```bash
mkdir -p outputs
```

3. **Start Python services (Docker)**
```bash
cd python
docker-compose up -d

# Check services are running
docker-compose ps
curl http://localhost:5000/health
```

4. **Build and run Rust app**
```bash
cd ..  # Back to project root
cargo build --release
cargo run --release
```

## 📖 Usage

### Text-to-3D Generation

1. Enter a text prompt in the sidebar (e.g., "a red sports car")
2. Click "🎨 Generate 3D Model"
3. Wait for generation to complete (~30-60 seconds)
4. Interact with the generated 3D model

**Example Prompts:**
- `a yellow rubber duck`
- `a blue crystal gem`
- `a wooden chair`
- `a futuristic spaceship`
- `a medieval sword`
- `a coffee mug`

### Camera Controls

- **Rotate**: Left-click and drag
- **Zoom**: Mouse wheel
- **Reset**: Click "🔄 Reset Camera" button

## 🏛️ Project Structure

```
.
├── outputs/              # Generated .ply files (shared between Docker and Rust)
│
├── crates/
│   ├── gj-app/           # Main application
│   │   ├── src/
│   │   │   ├── app.rs    # Application entry point
│   │   │   ├── state.rs  # Application state management
│   │   │   ├── ui/       # UI components (egui)
│   │   │   ├── events.rs # Event system
│   │   │   ├── gfx.rs    # Graphics state (wgpu)
│   │   │   └── worker.rs # Python service client
│   │   └── Cargo.toml
│   │
│   ├── gj-core/          # Core data structures
│   │   ├── src/
│   │   │   ├── gaussian_cloud.rs  # Gaussian splat data
│   │   │   ├── model_types.rs     # Model type definitions
│   │   │   └── error.rs           # Error types
│   │   └── Cargo.toml
│   │
│   └── gj-splat/         # Gaussian splatting renderer
│       ├── src/
│       │   ├── renderer.rs        # Main renderer
│       │   └── camera.rs          # Camera controller
│       ├── shaders/
│       │   └── gaussian.wgsl      # Optimized shader
│       └── Cargo.toml
│
├── python/
│   ├── docker-compose.yml  # Service orchestration
│   │
│   ├── api/                # FastAPI REST API
│   │   ├── main.py         # Job submission and status
│   │   ├── Dockerfile
│   │   └── requirements.txt
│   │
│   ├── worker/             # Celery worker
│   │   ├── worker.py       # Task processing
│   │   ├── Dockerfile
│   │   └── requirements.txt
│   │
│   ├── shared/             # Shared modules
│   │   ├── config.py       # Configuration
│   │   └── celery_app.py   # Celery setup
│   │
│   ├── models/             # Model implementations
│   │   ├── model.py        # Base class
│   │   └── shap_e.py       # Shap-E implementation
│   │
│   └── redis/
│       └── redis.conf      # Redis configuration
│
├── Cargo.toml            # Workspace configuration
└── README.md
```

## 🔧 Configuration

### Rendering Performance

Adjust these in `crates/gj-splat/src/renderer.rs`:

```rust
// Opacity threshold (higher = fewer splats, faster)
opacity > 0.1  // Default: 0.1

// Scale multiplier (smaller = smaller splats)
scale * 0.5    // Default: 0.5

// Opacity multiplier (lower = more transparent)
opacity * 0.4  // Default: 0.4
```

### Shap-E Settings

In `python/shared/config.py`:

```python
DEFAULT_GUIDANCE_SCALE = 15.0       # Higher = more prompt adherence
DEFAULT_NUM_INFERENCE_STEPS = 64    # More steps = better quality
```

## 🐛 Troubleshooting

### Services won't start

```bash
# Check Docker is running
docker ps

# Check service logs
cd python
docker-compose logs api
docker-compose logs worker

# Restart services
docker-compose down
docker-compose up -d
```

**Common fixes:**

1. **Wrong volume mount** - Check `python/docker-compose.yml`:
   ```yaml
   volumes:
     - ../outputs:/app/outputs  # Correct
   # NOT:
     - ./outputs:/app/outputs   # Wrong
   ```

2. **Missing outputs directory**:
   ```bash
   mkdir -p outputs
   ```

3. **Rebuild after fixes**:
   ```bash
   cd python
   docker-compose down
   docker-compose up -d
   cd ..
   cargo build --release
   ```

### Slow rendering / Low FPS

- **Reduce Gaussian count**: Increase opacity threshold in renderer
- **Lower resolution**: Reduce window size
- **Update GPU drivers**: Ensure latest drivers installed

### API connection failed

```bash
# Check API is running
curl http://localhost:5000/health

# Should return:
# {"status": "healthy", "workers": 1, ...}

# If not, check logs
docker-compose logs api
```

## 📊 Performance Benchmarks

| Operation | Time (GPU) | Time (CPU) |
|-----------|------------|------------|
| Shap-E generation | ~30-60s | ~5-10min |
| Load .ply file | ~100ms | ~200ms |
| Render frame (50k splats) | ~16ms | ~100ms |

*Tested on RTX 2060 Ti, i7-9750H*

## 🎓 Technical Details

### Gaussian Splatting

This project uses **3D Gaussian Splatting** for rendering, which represents scenes as collections of 3D Gaussians with:
- **Position**: 3D center point
- **Scale**: Size along each axis
- **Rotation**: Quaternion orientation
- **Color**: RGB appearance
- **Opacity**: Transparency

Each Gaussian is rendered as a textured quad with Gaussian falloff, blended using alpha compositing.

### Shap-E Model

**Shap-E** is OpenAI's text-to-3D model that:
1. Takes text prompt as input
2. Generates a latent 3D representation (~30-60 seconds)
3. Decodes to mesh/point cloud
4. Converts to Gaussian splats for rendering

**Key advantages:**
- Fast inference (~1 minute vs 5+ minutes for other models)
- Direct text-to-3D (no intermediate images needed)
- Produces clean, coherent 3D objects

### Asynchronous Architecture

- **Main thread**: UI and rendering (60 FPS target)
- **Worker thread**: API communication and file I/O
- **Python services**: GPU-accelerated 3D generation (Docker)
- **Redis**: Message queue and result storage
- **Celery**: Task management and progress tracking

This ensures the UI remains responsive during generation.

## 🔮 Local Development (Without Docker)

If you prefer to run Python services locally:

```bash
# Install Conda
# Then:
cd python
bash setup_local.sh

# Terminal 1 - Redis
redis-server redis/redis.conf

# Terminal 2 - Worker
conda activate genjutsu
cd worker
python worker.py

# Terminal 3 - API
conda activate genjutsu
cd api
uvicorn main:app --reload

# Terminal 4 - Rust app
cd ../..
cargo run --release
```

See `python/setup_local.sh` for full setup instructions.

## 📝 API Documentation

Once services are running, visit:
- **Swagger UI**: http://localhost:5000/docs
- **ReDoc**: http://localhost:5000/redoc

### Key Endpoints

```bash
# Health check
GET /health

# Submit generation job
POST /generate
{
  "prompt": "a red car",
  "model": "shap_e",
  "guidance_scale": 15.0,
  "num_inference_steps": 64
}

# Check job status
GET /status/{job_id}

# List active workers
GET /workers
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📝 License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Shap-E**: OpenAI's text-to-3D model ([Paper](https://arxiv.org/abs/2305.02463))
- **3D Gaussian Splatting**: Original rendering technique ([Paper](https://repo-sam.inria.fr/fungraph/3d-gaussian-splatting/))
- **egui**: Immediate mode GUI framework
- **wgpu**: WebGPU implementation in Rust

## 📚 Additional Resources

- [Shap-E Paper](https://arxiv.org/abs/2305.02463)
- [3D Gaussian Splatting Paper](https://repo-sam.inria.fr/fungraph/3d-gaussian-splatting/)
- [Project Documentation](docs/)
- [API Reference](http://localhost:5000/docs)

## ⚠️ Known Limitations

- Shap-E works best for **single objects**, not complex scenes
- Quality depends heavily on prompt clarity
- Generation time: ~30-60 seconds per object
- Best results with concrete, describable objects
- Abstract concepts may produce unexpected results

## 🗺️ Roadmap

- [ ] Support for more 3D generation models
- [ ] Export to common 3D formats (OBJ, FBX, GLTF)
- [ ] Texture and material editing
- [ ] Animation support
- [ ] Web-based viewer