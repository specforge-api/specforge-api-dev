"""FastAPI application demonstrating zero-code integration with SpecForge Dev container."""

from fastapi import FastAPI, HTTPException, status
from pydantic import BaseModel, EmailStr, Field

app = FastAPI(
    title="FastAPI Store Service",
    version="1.0.0",
    description="Backend service running alongside SpecForge Dev container",
)

class UserCreate(BaseModel):
    username: str = Field(..., min_length=3, max_length=20)
    email: EmailStr
    age: int = Field(..., ge=18, le=120)

class UserResponse(UserCreate):
    id: int

_DB: list[UserResponse] = []

@app.post("/api/v1/users", response_model=UserResponse, status_code=status.HTTP_201_CREATED)
def create_user(payload: UserCreate) -> UserResponse:
    # Explicit whitespace check to demonstrate boundary resilience
    if not payload.username.strip():
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Username cannot be whitespace-only",
        )
    user = UserResponse(id=len(_DB) + 1, **payload.model_dump())
    _DB.append(user)
    return user

@app.get("/api/v1/users", response_model=list[UserResponse])
def list_users() -> list[UserResponse]:
    return _DB

