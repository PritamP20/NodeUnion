# Docker deployment for NodeUnion Dashboard

FROM node:20-alpine AS builder

WORKDIR /app

# Install dependencies
COPY package*.json ./
RUN npm ci

# Build
COPY . .
RUN npm run build

# Production image
FROM node:20-alpine

WORKDIR /app

# Install serve to run the app
RUN npm install -g serve

# Copy built app from builder
COPY --from=builder /app/dist ./dist

# Copy env example
COPY .env.example .env.example

EXPOSE 5173

CMD ["serve", "-s", "dist", "-l", "5173"]
