#!/bin/bash
set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║        Genjutsu - Local Setup (Conda)                   ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Check if conda is installed
if ! command -v conda &> /dev/null; then
    echo "❌ Conda not found!"
    echo "Please install Miniconda or Anaconda first:"
    echo "  https://docs.conda.io/en/latest/miniconda.html"
    exit 1
fi

echo "✓ Conda found: $(conda --version)"
echo ""

# Create conda environment
echo "Creating conda environment 'genjutsu'..."
if conda env list | grep -q "^genjutsu "; then
    echo "Environment 'genjutsu' already exists."
    read -p "Do you want to remove and recreate it? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        conda env remove -n genjutsu -y
    else
        echo "Keeping existing environment."
        echo ""
        echo "To activate: conda activate genjutsu"
        echo "To update dependencies: conda activate genjutsu && pip install -r requirements.txt"
        exit 0
    fi
fi

# Create environment with Python 3.12 and PyTorch
echo "Creating environment with Python 3.12 and PyTorch (CUDA 12.4)..."
conda create -n genjutsu python=3.12 -y

# Activate environment
eval "$(conda shell.bash hook)"
conda activate genjutsu

# Install PyTorch with CUDA 12.4
echo ""
echo "Installing PyTorch with CUDA 12.4..."
pip install torch==2.5.1 torchvision==0.20.1 --index-url https://download.pytorch.org/whl/cu124

# Install other dependencies
echo ""
echo "Installing other dependencies..."
pip install -r requirements.txt

# Clone and install Shap-E
echo ""
echo "Installing Shap-E..."
if [ -d "shap-e" ]; then
    echo "Shap-E directory already exists, skipping clone..."
else
    git clone https://github.com/openai/shap-e.git
fi
cd shap-e
pip install -e .
cd ..

# Create outputs directory
mkdir -p ../outputs

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║  ✓ Setup complete!                                       ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "Next steps:"
echo "  1. Activate environment:"
echo "     conda activate genjutsu"
echo ""
echo "  2. Start the Python service:"
echo "     cd python"
echo "     python multi_model_service.py"
echo ""
echo "  3. In another terminal, build and run the Rust app:"
echo "     cargo run --release"
echo ""
echo "  4. Test the service:"
echo "     curl http://localhost:5000/health"
echo ""