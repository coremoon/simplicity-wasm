# Simplicity Compiler - Python/Flask Backend
FROM python:3.11-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends curl && \
    rm -rf /var/lib/apt/lists/*

# Copy Python requirements
COPY ./python/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy WASM dist files
COPY ./dist ./dist

# Copy Flask app
COPY ./python/python_app.py ./app.py

RUN useradd -m -u 1000 app && chown -R app:app /app
USER app

EXPOSE 5000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl --fail http://localhost:5000/api/health || exit 1

CMD ["python", "app.py"]
