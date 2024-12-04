FROM node:20@sha256:f4755c9039bdeec5c736b2e0dd5b47700d6393b65688b9e9f807ec12f54a8690
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
