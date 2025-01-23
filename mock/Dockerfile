FROM node:20@sha256:bc3d86568d9a9e062cdf7036367f6e2ce201925e2912758bcd3b0e2657875a63
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
