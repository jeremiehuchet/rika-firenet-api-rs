FROM node:24@sha256:33cf7f057918860b043c307751ef621d74ac96f875b79b6724dcebf2dfd0db6d
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
