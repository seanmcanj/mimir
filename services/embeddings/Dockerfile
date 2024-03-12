ARG python=python:3.12-slim

FROM ${python} AS build

RUN apt-get update \
    && apt-get install --assume-yes --no-install-recommends --quiet \
    ca-certificates \
    cmake \
    git \
    g++ \
    make \
    libzip-dev \
    && apt-get clean all

RUN python3 -m venv /venv
ENV PATH=/venv/bin:$PATH

WORKDIR /app
RUN pip install fastembed fastapi uvicorn pydantic


FROM ${python}

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive \
    apt-get install --no-install-recommends --assume-yes \
    libpq5

# Copy the virtual environment from the first stage.
COPY --from=build /venv /venv
ENV PATH=/venv/bin:$PATH

# Copy the application in.
COPY . .
CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "3000"]