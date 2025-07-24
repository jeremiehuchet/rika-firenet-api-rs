FROM node:22@sha256:37ff334612f77d8f999c10af8797727b731629c26f2e83caa6af390998bdc49c
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
