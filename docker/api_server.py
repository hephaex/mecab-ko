#!/usr/bin/env python3
"""
MeCab-Ko FastAPI Server

A production-ready REST API for Korean morphological analysis using mecab-ko-python.

Endpoints:
    GET /health - Health check
    GET /info - Server information
    POST /analyze - Analyze Korean text
    POST /morphs - Extract morphemes
    POST /nouns - Extract nouns
    POST /pos - Part-of-speech tagging
    POST /batch - Batch analysis
"""

import logging
import os
from typing import Optional

from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse
from mecab_ko import Mecab
from pydantic import BaseModel, Field

# ============================================================================
# Configuration
# ============================================================================

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Application metadata
APP_VERSION = "0.5.0"
APP_NAME = "MeCab-Ko API Server"
DICTIONARY_PATH = os.environ.get("MECAB_DIC_DIR", "/usr/share/mecab-ko-dic")

# ============================================================================
# Request/Response Models
# ============================================================================

class AnalysisRequest(BaseModel):
    """Text analysis request"""
    text: str = Field(..., min_length=1, max_length=10000, description="Korean text to analyze")
    format: Optional[str] = Field(
        default="default",
        description="Output format: default, json, csv"
    )


class MorphemeInfo(BaseModel):
    """Morpheme information"""
    surface: str = Field(..., description="Surface form")
    pos: str = Field(..., description="Part-of-speech tag")


class AnalysisResponse(BaseModel):
    """Analysis response"""
    success: bool
    data: dict = Field(..., description="Analysis result")
    error: Optional[str] = None
    processing_time_ms: Optional[float] = None


class BatchAnalysisRequest(BaseModel):
    """Batch analysis request"""
    texts: list[str] = Field(..., min_items=1, max_items=1000, description="List of Korean texts")
    format: Optional[str] = Field(default="default", description="Output format")


class BatchAnalysisResponse(BaseModel):
    """Batch analysis response"""
    success: bool
    results: list[dict]
    failed: int = 0
    total: int
    processing_time_ms: Optional[float] = None


class HealthResponse(BaseModel):
    """Health check response"""
    status: str
    version: str
    dictionary_path: str


class InfoResponse(BaseModel):
    """Server information response"""
    name: str
    version: str
    dictionary_path: str
    python_implementation: str
    description: str

# ============================================================================
# FastAPI Application
# ============================================================================

app = FastAPI(
    title=APP_NAME,
    description="REST API for Korean morphological analysis using MeCab-Ko",
    version=APP_VERSION,
    docs_url="/docs",
    redoc_url="/redoc",
    openapi_url="/openapi.json"
)

# Global tokenizer instance (lazy loaded)
_mecab: Optional[Mecab] = None


def get_mecab() -> Mecab:
    """Get or initialize Mecab tokenizer instance"""
    global _mecab
    if _mecab is None:
        logger.info(f"Initializing Mecab with dictionary: {DICTIONARY_PATH}")
        try:
            _mecab = Mecab()
            logger.info("Mecab initialized successfully")
        except Exception as e:
            logger.error(f"Failed to initialize Mecab: {e}")
            raise HTTPException(status_code=500, detail="Failed to initialize Mecab tokenizer")
    return _mecab

# ============================================================================
# Health & Info Endpoints
# ============================================================================

@app.get("/health", response_model=HealthResponse, tags=["Health"])
async def health_check() -> HealthResponse:
    """Health check endpoint"""
    try:
        mecab = get_mecab()
        return HealthResponse(
            status="healthy",
            version=APP_VERSION,
            dictionary_path=DICTIONARY_PATH
        )
    except Exception as e:
        logger.error(f"Health check failed: {e}")
        raise HTTPException(status_code=503, detail="Service unavailable")


@app.get("/info", response_model=InfoResponse, tags=["Info"])
async def server_info() -> InfoResponse:
    """Get server information"""
    return InfoResponse(
        name=APP_NAME,
        version=APP_VERSION,
        dictionary_path=DICTIONARY_PATH,
        python_implementation="CPython with Rust extension (mecab-ko-python)",
        description="Korean morphological analyzer using MeCab-Ko"
    )

# ============================================================================
# Analysis Endpoints
# ============================================================================

@app.post("/analyze", response_model=AnalysisResponse, tags=["Analysis"])
async def analyze(request: AnalysisRequest) -> AnalysisResponse:
    """
    Analyze Korean text using MeCab-Ko

    Returns morphological analysis in specified format.
    """
    import time

    try:
        start_time = time.time()
        mecab = get_mecab()

        text = request.text.strip()
        if not text:
            raise ValueError("Text cannot be empty")

        # Perform analysis
        result = mecab.parse(text)

        processing_time = (time.time() - start_time) * 1000

        return AnalysisResponse(
            success=True,
            data={"result": result},
            processing_time_ms=processing_time
        )
    except ValueError as e:
        logger.warning(f"Validation error: {e}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"Analysis error: {e}")
        raise HTTPException(status_code=500, detail="Analysis failed")


@app.post("/morphs", response_model=AnalysisResponse, tags=["Analysis"])
async def extract_morphemes(request: AnalysisRequest) -> AnalysisResponse:
    """
    Extract morphemes (tokens) from Korean text

    Returns list of morphemes.
    """
    import time

    try:
        start_time = time.time()
        mecab = get_mecab()

        text = request.text.strip()
        if not text:
            raise ValueError("Text cannot be empty")

        # Extract morphemes
        morphemes = mecab.morphs(text)

        processing_time = (time.time() - start_time) * 1000

        return AnalysisResponse(
            success=True,
            data={"morphemes": morphemes, "count": len(morphemes)},
            processing_time_ms=processing_time
        )
    except ValueError as e:
        logger.warning(f"Validation error: {e}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"Morphs extraction error: {e}")
        raise HTTPException(status_code=500, detail="Morphs extraction failed")


@app.post("/nouns", response_model=AnalysisResponse, tags=["Analysis"])
async def extract_nouns(request: AnalysisRequest) -> AnalysisResponse:
    """
    Extract nouns from Korean text

    Returns list of nouns.
    """
    import time

    try:
        start_time = time.time()
        mecab = get_mecab()

        text = request.text.strip()
        if not text:
            raise ValueError("Text cannot be empty")

        # Extract nouns
        nouns = mecab.nouns(text)

        processing_time = (time.time() - start_time) * 1000

        return AnalysisResponse(
            success=True,
            data={"nouns": nouns, "count": len(nouns)},
            processing_time_ms=processing_time
        )
    except ValueError as e:
        logger.warning(f"Validation error: {e}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"Nouns extraction error: {e}")
        raise HTTPException(status_code=500, detail="Nouns extraction failed")


@app.post("/pos", response_model=AnalysisResponse, tags=["Analysis"])
async def pos_tagging(request: AnalysisRequest) -> AnalysisResponse:
    """
    Perform part-of-speech tagging on Korean text

    Returns list of (morpheme, POS tag) tuples.
    """
    import time

    try:
        start_time = time.time()
        mecab = get_mecab()

        text = request.text.strip()
        if not text:
            raise ValueError("Text cannot be empty")

        # POS tagging
        pos_tags = mecab.pos(text)

        # Convert to JSON-serializable format
        pos_data = [
            {"surface": morpheme, "pos": pos}
            for morpheme, pos in pos_tags
        ]

        processing_time = (time.time() - start_time) * 1000

        return AnalysisResponse(
            success=True,
            data={"pos_tags": pos_data, "count": len(pos_data)},
            processing_time_ms=processing_time
        )
    except ValueError as e:
        logger.warning(f"Validation error: {e}")
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        logger.error(f"POS tagging error: {e}")
        raise HTTPException(status_code=500, detail="POS tagging failed")


@app.post("/batch", response_model=BatchAnalysisResponse, tags=["Analysis"])
async def batch_analyze(request: BatchAnalysisRequest) -> BatchAnalysisResponse:
    """
    Batch analysis of multiple Korean texts

    Analyzes multiple texts efficiently in one request.
    """
    import time

    try:
        start_time = time.time()
        mecab = get_mecab()

        results = []
        failed = 0

        for text in request.texts:
            try:
                text = text.strip()
                if not text:
                    results.append({"error": "Empty text"})
                    failed += 1
                    continue

                result = mecab.parse(text)
                results.append({"success": True, "result": result})
            except Exception as e:
                logger.error(f"Batch analysis error for text: {e}")
                results.append({"success": False, "error": str(e)})
                failed += 1

        processing_time = (time.time() - start_time) * 1000

        return BatchAnalysisResponse(
            success=failed == 0,
            results=results,
            failed=failed,
            total=len(request.texts),
            processing_time_ms=processing_time
        )
    except Exception as e:
        logger.error(f"Batch analysis error: {e}")
        raise HTTPException(status_code=500, detail="Batch analysis failed")

# ============================================================================
# Middleware & Error Handling
# ============================================================================

@app.middleware("http")
async def add_process_time_header(request: Request, call_next):
    """Add processing time to response headers"""
    import time
    start_time = time.time()
    response = await call_next(request)
    process_time = time.time() - start_time
    response.headers["X-Process-Time"] = str(process_time)
    return response


@app.exception_handler(Exception)
async def general_exception_handler(request: Request, exc: Exception):
    """Handle uncaught exceptions"""
    logger.error(f"Unhandled exception: {exc}")
    return JSONResponse(
        status_code=500,
        content={
            "success": False,
            "error": "Internal server error",
            "detail": str(exc) if os.environ.get("DEBUG") else None
        }
    )

# ============================================================================
# Root Endpoint
# ============================================================================

@app.get("/", tags=["Root"])
async def root():
    """Root endpoint with API information"""
    return {
        "name": APP_NAME,
        "version": APP_VERSION,
        "docs": "/docs",
        "openapi": "/openapi.json",
        "health": "/health",
        "info": "/info"
    }

# ============================================================================
# Startup/Shutdown Events
# ============================================================================

@app.on_event("startup")
async def startup_event():
    """Initialize resources on startup"""
    logger.info(f"{APP_NAME} v{APP_VERSION} starting up")
    try:
        get_mecab()
        logger.info("Server ready to handle requests")
    except Exception as e:
        logger.error(f"Startup failed: {e}")
        raise


@app.on_event("shutdown")
async def shutdown_event():
    """Clean up resources on shutdown"""
    logger.info(f"{APP_NAME} shutting down")


if __name__ == "__main__":
    import uvicorn

    # Development server
    uvicorn.run(
        "api_server:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
        log_level="info"
    )
