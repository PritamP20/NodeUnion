const http = require('http');
const os = require('os');

const hostname = '0.0.0.0';
const port = 3000;

const server = http.createServer((req, res) => {
  const uptime = Math.floor(process.uptime());
  const memory = Math.round(process.memoryUsage().heapUsed / 1024 / 1024);
  
  const response = {
    status: "running",
    app: "NodeUnion Test Node",
    timestamp: new Date().toISOString(),
    uptime_seconds: uptime,
    memory_mb: memory,
    hostname: os.hostname(),
    platform: os.platform(),
    cpus: os.cpus().length,
    node_version: process.version,
    env: {
      NODE_ENV: process.env.NODE_ENV || 'development',
      JOB_ID: process.env.JOB_ID || 'unknown',
      CHUNK_ID: process.env.CHUNK_ID || 'unknown'
    }
  };
  
  res.statusCode = 200;
  res.setHeader('Content-Type', 'application/json');
  res.end(JSON.stringify(response, null, 2));
});

server.listen(port, hostname, () => {
  console.log(`NodeUnion node running at http://${hostname}:${port}/`);
  console.log(`Process ID: ${process.pid}`);
  console.log(`Uptime: ${Math.floor(process.uptime())}s`);
});

// Handle shutdown gracefully
process.on('SIGTERM', () => {
  console.log('SIGTERM received, shutting down gracefully...');
  server.close(() => {
    console.log('Server closed');
    process.exit(0);
  });
  setTimeout(() => {
    console.error('Forcefully shutting down');
    process.exit(1);
  }, 10000);
});
