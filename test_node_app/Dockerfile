FROM node:20-alpine

WORKDIR /app

# Copy the app
COPY app.js .

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=10s --timeout=5s --start-period=5s --retries=3 \
  CMD node -e "require('http').get('http://127.0.0.1:3000/', r => {if(r.statusCode!==200) throw new Error(r.statusCode)})"

# Start the app
CMD ["node", "app.js"]
