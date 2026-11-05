FROM node:22@sha256:e273cf2a19c7334c03f52e2ca933ecdfc3fbfaad49db7b8bc8c3edd9ac34e9c7
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
