FROM node:22@sha256:7e791fc54bd02fc89fd4fb39eb37e5bea753c75679c8022478d81679367d995a
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
