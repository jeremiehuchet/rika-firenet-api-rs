FROM node:22@sha256:ecabd1cb6956d7acfffe8af6bbfbe2df42362269fd28c227f36367213d0bb777
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
