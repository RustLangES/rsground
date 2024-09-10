# Building layer
FROM node:alpine3.20 AS build-runner
WORKDIR /tmp/frontend
COPY package*.json ./
RUN npm i --ignore-scripts
COPY src ./src
COPY tsconfig.json .
COPY tsconfig.node.json .
COPY tsconfig.app.json .
COPY vite.config.ts .
COPY index.html .
RUN npm run build

## Running layer
FROM node:alpine3.20 AS prod-runner
WORKDIR /frontend
COPY --from=build-runner /tmp/frontend/package*.json ./
RUN npm i --omit=dev --ignore-scripts
COPY --from=build-runner /tmp/frontend/dist ./dist
RUN npm i -g serve

ENV NODE_ENV=production
EXPOSE ${PORT}

USER node
CMD ["serve", "-s", "dist", "-l", "5173"]
