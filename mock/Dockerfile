FROM node:24@sha256:14078204b6bb268c107ceff6d3c43f45a888105e5c2b18b14c32b3409d93a1b0
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
