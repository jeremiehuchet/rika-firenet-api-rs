FROM node:18@sha256:b08b1356559e2e9945f47ded630c9eb9d4e3ca04f111ab20d55652442cf14aee
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
