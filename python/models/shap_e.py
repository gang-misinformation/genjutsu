"""
Shap-E: Text-to-3D generation using OpenAI's Shap-E model
Fast inference (~1 minute) with high-quality Gaussian splat output
"""

import torch
import numpy as np
from pathlib import Path
from PIL import Image

from .model import Model3DBase


class ShapEModel(Model3DBase):
    """OpenAI Shap-E: Fast text-to-3D generation"""

    def __init__(self, device="cuda"):
        super().__init__(device)
        self.text_model = None
        self.diffusion_model = None

    def load(self) -> bool:
        """Load Shap-E models"""
        try:
            print("  Loading Shap-E models...")

            from shap_e.diffusion.sample import sample_latents
            from shap_e.diffusion.gaussian_diffusion import diffusion_from_config
            from shap_e.models.download import load_model, load_config
            from shap_e.util.notebooks import decode_latent_mesh

            # Store functions we'll need
            self.sample_latents = sample_latents
            self.diffusion_from_config = diffusion_from_config
            self.load_model = load_model
            self.load_config = load_config
            self.decode_latent_mesh = decode_latent_mesh

            # Load text-to-latent model
            print("    Loading text encoder...")
            self.text_model = load_model('text300M', device=self.device)

            # Load latent diffusion model
            print("    Loading diffusion model...")
            self.diffusion_model = load_model('transmitter', device=self.device)

            self.is_loaded = True
            print("  ✓ Shap-E loaded successfully")
            return True

        except Exception as e:
            print(f"  ✗ Failed to load Shap-E: {e}")
            print(f"  → Make sure you installed: pip install git+https://github.com/openai/shap-e.git")
            return False

    def generate(self, prompt: str, output_path: Path, **kwargs) -> Path:
        """Generate 3D Gaussians from text prompt"""
        if not self.is_loaded:
            raise RuntimeError("Shap-E not loaded")

        guidance_scale = kwargs.get('guidance_scale', 15.0)
        num_inference_steps = kwargs.get('num_inference_steps', 64)

        print(f"  Generating with Shap-E: '{prompt}'")
        print(f"  Guidance scale: {guidance_scale}")
        print(f"  Inference steps: {num_inference_steps}")

        # Generate latents
        print("  [1/2] Generating latent representation...")
        latents = self.sample_latents(
            batch_size=1,
            model=self.text_model,
            diffusion=self.diffusion_from_config(self.load_config('diffusion')),
            guidance_scale=guidance_scale,
            model_kwargs=dict(texts=[prompt]),
            progress=True,
            clip_denoised=True,
            use_fp16=True,
            use_karras=True,
            karras_steps=num_inference_steps,
            sigma_min=1e-3,
            sigma_max=160,
            s_churn=0,
        )

        # Decode to mesh/point cloud
        print("  [2/2] Decoding to 3D representation...")

        # Extract point cloud with colors from latent
        pc = self.decode_latent_mesh(self.diffusion_model, latents[0]).tri_mesh()

        # Convert to Gaussian splats
        self._export_to_ply(pc, output_path)

        print(f"  ✓ Saved to {output_path}")
        return output_path

    def get_name(self) -> str:
        return "Shap-E"

    def get_estimated_time(self, **kwargs) -> int:
        """Returns time in seconds"""
        num_steps = kwargs.get('num_inference_steps', 64)
        return 30 + (num_steps * 0.5)  # ~30-60 seconds

    def _export_to_ply(self, mesh, output_path):
        """Convert Shap-E mesh to PLY format with Gaussian splat data"""
        import trimesh

        # Get vertices and colors
        vertices = mesh.verts
        colors = mesh.vertex_channels.get('color', np.ones_like(vertices))

        # Convert colors from [0,1] to [0,255]
        if colors.max() <= 1.0:
            colors = (colors * 255).astype(np.uint8)

        num_points = len(vertices)

        # Create Gaussian parameters
        # Each point becomes a small Gaussian splat
        scales = np.ones((num_points, 3), dtype=np.float32) * 0.05  # Small scale
        rotations = np.zeros((num_points, 4), dtype=np.float32)
        rotations[:, 0] = 1.0  # Identity quaternion (w=1, x=0, y=0, z=0)
        opacities = np.ones(num_points, dtype=np.float32) * 0.8

        # Write PLY file
        with open(output_path, 'wb') as f:
            # Header
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

            # Data
            for i in range(num_points):
                # Position
                f.write(vertices[i].astype(np.float32).tobytes())

                # Normal (placeholder)
                f.write(np.zeros(3, dtype=np.float32).tobytes())

                # Color (RGB as uint8)
                f.write(colors[i].astype(np.uint8).tobytes())

                # Opacity
                f.write(opacities[i].astype(np.float32).tobytes())

                # Scale
                f.write(scales[i].astype(np.float32).tobytes())

                # Rotation (quaternion)
                f.write(rotations[i].astype(np.float32).tobytes())