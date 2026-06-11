# Kronos Forecasting Service for ATES

This is a lightweight FastAPI microservice that exposes the Kronos foundation model as a REST API.

## How to Run

1. First time setup (download model):
   ```bash
   # Option A: Let it auto-download from Hugging Face
   python -m uvicorn main:app --host 0.0.0.0 --port 8000

   # Option B: Use locally downloaded models (recommended for full control)
   export KRONOS_MODEL_PATH=./models/Kronos-small
   export KRONOS_TOKENIZER_PATH=./models/Kronos-Tokenizer-base
   python -m uvicorn main:app --host 0.0.0.0 --port 8000
   ```

2. The service will be available at http://localhost:8000

3. Test the endpoint:
   POST http://localhost:8000/forecast

## Important Notes
- The `model/` folder from the original Kronos repository must be available in the Python path (or copy it here).
- For full offline use, download the models once using `huggingface-cli download NeoQuasar/Kronos-small --local-dir ./models/Kronos-small`
- This service is designed to be called by the ATES Rust orchestrator.

## Integration with ATES
The Rust side (kronos_client.rs) calls this service over HTTP.
