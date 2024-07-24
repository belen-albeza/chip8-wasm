/// <reference types="vite/client" />
declare module "*.svg" {
  const content: SVGElement;
  export default content;
}
