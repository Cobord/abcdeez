from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="abcdeez",
    version="0.1.0",
    author="Graph Learning Research",
    author_email="research@graphlearning.app",
    description="Python client for the Graph Learning System (abcdeez)",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/yourusername/abcdeez",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Science/Research",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
    python_requires=">=3.8",
    install_requires=[
        "requests>=2.28.0",
        "websocket-client>=1.0.0",
        "pandas>=1.5.0",
        "numpy>=1.21.0",
        "matplotlib>=3.5.0",
        "seaborn>=0.12.0",
        "pydantic>=2.0.0",
        "python-dateutil>=2.8.0",
        "pyarrow>=10.0.0",  # For Parquet support
    ],
    extras_require={
        "dev": [
            "pytest>=7.0",
            "pytest-asyncio>=0.21.0",
            "pytest-cov>=4.0",
            "black>=22.0",
            "flake8>=5.0",
            "mypy>=1.0",
            "sphinx>=5.0",
            "sphinx-rtd-theme>=1.0",
        ],
        "async": [
            "aiohttp>=3.8.0",
            "asyncio>=3.4.3",
        ],
    },
    entry_points={
        "console_scripts": [
            "abcdeez=abcdeez.cli:main",
        ],
    },
)