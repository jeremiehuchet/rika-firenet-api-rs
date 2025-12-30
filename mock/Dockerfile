FROM node:22@sha256:8739e532180cfe09e03bbb4545fc725b044c921280532d7c9c1480ba2396837e
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
