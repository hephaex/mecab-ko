#!/usr/bin/env python3
"""
FastAPI Server for Korean Morphological Analysis using mecab-ko-python

This REST API server provides endpoints for Korean text analysis including:
- Morpheme extraction
- Noun extraction
- Part-of-speech tagging
- Full morphological analysis

Dependencies:
    pip install fastapi uvicorn pydantic

Usage:
    python fastapi_server.py
    # Server runs at http://localhost:8000
    # API docs at http://localhost:8000/docs

Example requests:
    curl -X POST "http://localhost:8000/analyze" \\
         -H "Content-Type: application/json" \\
         -d '{"text": "안녕하세요"}'

    curl -X POST "http://localhost:8000/morphs" \\
         -H "Content-Type: application/json" \\
         -d '{"text": "자연어 처리는 재미있습니다"}'
"""

from typing import List, Tuple, Optional
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field, field_validator
import uvicorn
from mecab_ko import Mecab


# Request/Response Models
class TextRequest(BaseModel):
    """Request model for text analysis."""

    text: str = Field(
        ...,
        min_length=1,
        max_length=10000,
        description="Korean text to analyze"
    )

    @field_validator('text')
    @classmethod
    def validate_text(cls, v: str) -> str:
        """Validate text is not empty after stripping."""
        if not v.strip():
            raise ValueError('Text cannot be empty')
        return v


class MorphsResponse(BaseModel):
    """Response model for morpheme extraction."""

    text: str = Field(..., description="Original input text")
    morphs: List[str] = Field(..., description="Extracted morphemes")
    count: int = Field(..., description="Number of morphemes")


class NounsResponse(BaseModel):
    """Response model for noun extraction."""

    text: str = Field(..., description="Original input text")
    nouns: List[str] = Field(..., description="Extracted nouns")
    count: int = Field(..., description="Number of nouns")


class PosResponse(BaseModel):
    """Response model for POS tagging."""

    text: str = Field(..., description="Original input text")
    pos: List[Tuple[str, str]] = Field(
        ...,
        description="List of (morpheme, POS tag) tuples"
    )
    count: int = Field(..., description="Number of morphemes")


class ParseResponse(BaseModel):
    """Response model for full morphological analysis."""

    text: str = Field(..., description="Original input text")
    parse_result: str = Field(..., description="MeCab format output")


class HealthResponse(BaseModel):
    """Health check response."""

    status: str = Field(..., description="Service status")
    version: str = Field(..., description="mecab-ko-python version")


class ErrorResponse(BaseModel):
    """Error response model."""

    error: str = Field(..., description="Error message")
    detail: Optional[str] = Field(None, description="Detailed error information")


# FastAPI Application
app = FastAPI(
    title="MeCab-Ko Korean Morphological Analysis API",
    description="REST API for Korean text analysis using mecab-ko-python",
    version="1.0.0",
    docs_url="/docs",
    redoc_url="/redoc"
)


# Global MeCab instance
mecab: Optional[Mecab] = None


@app.on_event("startup")
async def startup_event():
    """Initialize MeCab tokenizer on startup."""
    global mecab
    try:
        mecab = Mecab()
        print("MeCab tokenizer initialized successfully")
    except Exception as e:
        print(f"Failed to initialize MeCab: {e}")
        raise RuntimeError(f"MeCab initialization failed: {e}")


@app.on_event("shutdown")
async def shutdown_event():
    """Cleanup on shutdown."""
    global mecab
    mecab = None
    print("MeCab tokenizer cleaned up")


@app.get("/", response_model=dict)
async def root():
    """Root endpoint with API information."""
    return {
        "message": "MeCab-Ko Korean Morphological Analysis API",
        "docs": "/docs",
        "endpoints": {
            "health": "/health",
            "analyze": "/analyze",
            "morphs": "/morphs",
            "nouns": "/nouns",
            "pos": "/pos"
        }
    }


@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint."""
    if mecab is None:
        raise HTTPException(status_code=503, detail="MeCab not initialized")

    return HealthResponse(
        status="healthy",
        version="0.5.0"
    )


@app.post("/analyze", response_model=ParseResponse, responses={
    400: {"model": ErrorResponse},
    500: {"model": ErrorResponse}
})
async def analyze_text(request: TextRequest):
    """
    Full morphological analysis in MeCab format.

    Returns detailed morphological information for each token.
    """
    if mecab is None:
        raise HTTPException(status_code=503, detail="MeCab not initialized")

    try:
        parse_result = mecab.parse(request.text)
        return ParseResponse(
            text=request.text,
            parse_result=parse_result
        )
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Analysis failed: {str(e)}"
        )


@app.post("/morphs", response_model=MorphsResponse, responses={
    400: {"model": ErrorResponse},
    500: {"model": ErrorResponse}
})
async def extract_morphs(request: TextRequest):
    """
    Extract morphemes from Korean text.

    Returns a list of morphological units (형태소).
    """
    if mecab is None:
        raise HTTPException(status_code=503, detail="MeCab not initialized")

    try:
        morphs = mecab.morphs(request.text)
        return MorphsResponse(
            text=request.text,
            morphs=morphs,
            count=len(morphs)
        )
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Morpheme extraction failed: {str(e)}"
        )


@app.post("/nouns", response_model=NounsResponse, responses={
    400: {"model": ErrorResponse},
    500: {"model": ErrorResponse}
})
async def extract_nouns(request: TextRequest):
    """
    Extract nouns from Korean text.

    Returns a list of noun tokens (명사).
    """
    if mecab is None:
        raise HTTPException(status_code=503, detail="MeCab not initialized")

    try:
        nouns = mecab.nouns(request.text)
        return NounsResponse(
            text=request.text,
            nouns=nouns,
            count=len(nouns)
        )
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Noun extraction failed: {str(e)}"
        )


@app.post("/pos", response_model=PosResponse, responses={
    400: {"model": ErrorResponse},
    500: {"model": ErrorResponse}
})
async def tag_pos(request: TextRequest):
    """
    Part-of-speech tagging for Korean text.

    Returns a list of (morpheme, POS tag) tuples.

    Common POS tags:
    - NNG: 일반 명사 (Common noun)
    - NNP: 고유 명사 (Proper noun)
    - VV: 동사 (Verb)
    - VA: 형용사 (Adjective)
    - JX: 보조사 (Auxiliary particle)
    - JKS: 주격조사 (Subject particle)
    - EF: 종결 어미 (Final ending)
    """
    if mecab is None:
        raise HTTPException(status_code=503, detail="MeCab not initialized")

    try:
        pos_tags = mecab.pos(request.text)
        return PosResponse(
            text=request.text,
            pos=pos_tags,
            count=len(pos_tags)
        )
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"POS tagging failed: {str(e)}"
        )


if __name__ == "__main__":
    print("Starting MeCab-Ko FastAPI Server...")
    print("API documentation: http://localhost:8000/docs")
    print("Alternative docs: http://localhost:8000/redoc")

    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        log_level="info"
    )
