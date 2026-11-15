/**
 * Sample Express.js application verified against SpecForge Dev (Rust Engine).
 * Notice: ZERO SpecForge dependencies or code imports!
 */

const express = require('express');
const app = express();
const port = process.env.PORT || 3000;

app.use(express.json());

// In-memory user store
const users = [];

// Serve the OpenAPI specification
app.get('/openapi.json', (req, res) => {
  res.json({
    openapi: '3.0.0',
    info: {
      title: 'Express Store API',
      version: '1.0.0',
      description: 'Express.js backend tested with SpecForge Dev container'
    },
    paths: {
      '/api/v1/users': {
        post: {
          summary: 'Register a new user',
          requestBody: {
            required: true,
            content: {
              'application/json': {
                schema: {
                  type: 'object',
                  properties: {
                    username: { type: 'string', minLength: 3, maxLength: 20 },
                    email: { type: 'string', format: 'email' },
                    age: { type: 'integer', minimum: 18, maximum: 120 }
                  },
                  required: ['username', 'email']
                }
              }
            }
          },
          responses: {
            '201': { description: 'User created' },
            '422': { description: 'Validation failed' }
          }
        },
        get: {
          summary: 'List users',
          responses: {
            '200': { description: 'Success' }
          }
        }
      }
    }
  });
});

// User registration endpoint
app.post('/api/v1/users', (req, res) => {
  const { username, email, age } = req.body || {};

  // Strict boundary validations
  if (!username || typeof username !== 'string' || username.trim().length < 3) {
    return res.status(422).json({ error: 'Username must be at least 3 non-whitespace characters' });
  }

  if (!email || typeof email !== 'string' || !email.includes('@')) {
    return res.status(422).json({ error: 'Valid email address required' });
  }

  if (age !== undefined && (typeof age !== 'number' || age < 18 || age > 120)) {
    return res.status(422).json({ error: 'Age must be an integer between 18 and 120' });
  }

  const newUser = { id: users.length + 1, username, email, age: age || 25 };
  users.push(newUser);
  res.status(201).json(newUser);
});

app.get('/api/v1/users', (req, res) => {
  res.json(users);
});

app.listen(port, () => {
  console.log(`Express API running on http://localhost:${port}`);
  console.log(`OpenAPI Spec available at http://localhost:${port}/openapi.json`);
});

