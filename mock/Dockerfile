FROM node:22@sha256:83355e9c59b3201495ef828a92feb8b16f57f50f5ac125781f17439f12c62e82
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
