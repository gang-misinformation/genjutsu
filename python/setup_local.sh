#!/bin/bash
set -e

echo "╔══════════════════════════════════════════════════════════╗"
echo "║        Genjutsu - Local Conda Setup                      ║"
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
        echo "Keeping existing environment. Run 'conda activate genjutsu' to use it."
        exit 0
    fi
fi

conda env create -f environment.yml

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