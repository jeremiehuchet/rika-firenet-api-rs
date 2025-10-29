FROM node:22@sha256:22ab967fdf8d9ce32387d0c06ab2e32c6793d860bb93ccce6e27927ccdfeee6d
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
