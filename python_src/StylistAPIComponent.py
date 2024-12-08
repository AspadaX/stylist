import base64
from enum import Enum
from typing import Any, Dict, List

import httpx


class Gender(str, Enum):
    MALE = "Male"
    FEMALE = "Female"


class StylistAPIComponent:
    """Synchronous component for interacting with the Stylist API"""
    
    def __init__(self, base_url: str) -> None:
        self.base_url = base_url
        self.client = httpx.Client(base_url=base_url, timeout=10000000)

    def __encode_image(self, image_path: str) -> str:
        with open(image_path, "rb") as image_file:
            return base64.b64encode(image_file.read()).decode()

    def upload_clothes(self, name: str, gender: Gender, image_path: str) -> Dict[str, Any]:
        payload = {
            "name": name,
            "gender": gender.value,
            "image": self.__encode_image(image_path)
        }
        
        response = self.client.post("/api/clothes/upload", json=payload)
        response.raise_for_status()
        return response.json()

    def get_clothes(self) -> List[Dict[str, Any]]:
        response = self.client.get("/api/clothes/get")
        response.raise_for_status()
        return response.json()

    def delete_clothes(self, clothes_id: str) -> Dict[str, Any]:
        response = self.client.delete(f"/api/clothes/delete/{clothes_id}")
        response.raise_for_status()
        return response.json()

    def calculate_similarity(self, image_path: str, top_n: int = 5) -> Dict[str, Any]:
        payload = {
            "user_image": self.__encode_image(image_path),
            "top_n": top_n
        }
        
        response = self.client.post("/api/similarity/calculate", json=payload)
        response.raise_for_status()
        return response.json()

    def save_store(self) -> Dict[str, Any]:
        response = self.client.get("/api/store/save")
        response.raise_for_status()
        return response.json()

    def load_store(self) -> Dict[str, Any]:
        response = self.client.get("/api/store/load")
        response.raise_for_status()
        return response.json()