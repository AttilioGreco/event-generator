# Build stage
FROM oven/bun:1-alpine AS dev-stage

# install dependencies
WORKDIR /app

# Copia solo i file di dipendenze per ottimizzare la cache
COPY package*.json bun.lock ./
ENV NODE_ENV=development
RUN bun install --frozen-lockfile --development

# Copy application
COPY . .

# Aggiungi variabile d'ambiente per disabilitare la cache durante lo sviluppo
ENV VITE_DISABLE_CACHE=true

COPY entrypoint.sh /usr/local/bin/entrypoint.sh
ENTRYPOINT ["entrypoint.sh"]
CMD ["bun", "dev"]