from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastembed import TextEmbedding
from pydantic import BaseModel
from typing import List
import json
import numpy as np


class EmbeddingRequest(BaseModel):
    docs: List[str]


embedding_model = TextEmbedding()
print("The model BAAI/bge-small-en-v1.5 is ready to use.")

app = FastAPI()
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


class NumpyEncoder(json.JSONEncoder):
    """Special json encoder for numpy types"""

    def default(self, obj):
        if isinstance(
            obj,
            (
                np.int_,
                np.intc,
                np.intp,
                np.int8,
                np.int16,
                np.int32,
                np.int64,
                np.uint8,
                np.uint16,
                np.uint32,
                np.uint64,
            ),
        ):
            return int(obj)
        elif isinstance(obj, (np.float_, np.float16, np.float32, np.float64)):
            return float(obj)
        elif isinstance(obj, (np.ndarray,)):
            return obj.tolist()
        return json.JSONEncoder.default(self, obj)


@app.get("/health", status_code=200)
async def healthcheck():
    return "OK"


@app.post("/get-embeddings")
async def create_embeddings(documents: EmbeddingRequest):
    embeddings_list = list(embedding_model.embed(documents.docs))[0].tolist()

    return json.dumps({"status": "OK", "embeddings": embeddings_list}, cls=NumpyEncoder)
