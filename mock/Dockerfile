FROM node:22@sha256:4ad2c2b350ab49fb637ab40a269ffe207c61818bb7eb3a4ea122001a0c605e1f
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
