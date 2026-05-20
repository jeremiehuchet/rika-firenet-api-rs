FROM node:22@sha256:1031993481795705055273f2eef0c24597abdcb277d6e058c82f78cbbdef92a6
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
