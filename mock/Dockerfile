FROM node:20@sha256:7eaecf12ba40043be19af57e2c7fba35d61f9b906128e3e4d11eda81b1ecb857
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
