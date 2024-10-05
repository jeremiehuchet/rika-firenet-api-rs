FROM node:20@sha256:fffa89e023a3351904c04284029105d9e2ac7020886d683775a298569591e5bb
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
