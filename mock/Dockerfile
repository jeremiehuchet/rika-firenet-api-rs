FROM node:22@sha256:dcf06103a9d4087e3244a51697adbbb85331dcb7161dbe994ca1cd07dd32e2a5
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
